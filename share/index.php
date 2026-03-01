<?php
declare(strict_types=1);
/**
 * DevPanel Welcome Page
 * Served from /var/www/html — the standard Apache webroot.
 * Reads virtual hosts from /etc/apache2/sites-available/devpanel.conf
 */

// ── Configuration ─────────────────────────────────────────────────────────
// This file lives at /var/www/html/index.php — the standard Apache webroot.
// Projects live in the first human user's ~/projects/ directory.
$DEVPANEL_CONF = '/etc/apache2/sites-available/devpanel.conf';
$SITES_ENABLED = '/etc/apache2/sites-enabled';

// Discover the projects directory from config.toml, or scan home dirs
function findProjectsRoot(): string {
    // Try to read from any human user's devpanel config
    $passwdLines = file('/etc/passwd') ?: [];
    foreach ($passwdLines as $line) {
        $parts = explode(':', trim($line));
        if (count($parts) < 7) continue;
        $uid  = (int)$parts[2];
        $home = $parts[5];
        if ($uid < 1000 || $uid >= 65534) continue;
        $cfg = "$home/.config/devpanel/config.toml";
        if (is_readable($cfg)) {
            foreach (file($cfg) as $cfgLine) {
                if (preg_match('/^repos_root\s*=\s*"([^"]+)"/', trim($cfgLine), $m)) {
                    return $m[1];
                }
            }
        }
        // Fallback: ~/projects if it exists
        if (is_dir("$home/projects")) return "$home/projects";
    }
    return '/var/www/html';
}

$PROJECTS_ROOT = findProjectsRoot();

// ── Gather projects ────────────────────────────────────────────────────────
$projects = [];
if (is_dir($PROJECTS_ROOT) && is_readable($PROJECTS_ROOT)) {
    foreach (scandir($PROJECTS_ROOT) as $entry) {
        if ($entry === '.' || $entry === '..' || str_starts_with($entry, '.')) continue;
        $full = $PROJECTS_ROOT . '/' . $entry;
        if (is_dir($full)) {
            $projects[] = $entry;
        }
    }
}

// ── Parse devpanel.conf for VirtualHosts ──────────────────────────────────
$vhosts  = [];
$issues  = [];

function parseDevpanelConf(string $path): array {
    $hosts = [];
    if (!is_readable($path)) return $hosts;

    $content   = file_get_contents($path);
    $inBlock   = false;
    $current   = [];

    foreach (explode("\n", $content) as $raw) {
        $line = trim($raw);
        if (stripos($line, '<VirtualHost') === 0) {
            $inBlock = true;
            $current = ['server_name' => '', 'document_root' => '', 'aliases' => []];
            continue;
        }
        if (stripos($line, '</VirtualHost>') === 0 && $inBlock) {
            if ($current['server_name'] !== '') $hosts[] = $current;
            $inBlock = false;
            continue;
        }
        if (!$inBlock) continue;

        if (preg_match('/^ServerName\s+(\S+)/i', $line, $m))      $current['server_name']   = $m[1];
        if (preg_match('/^DocumentRoot\s+(\S+)/i', $line, $m))     $current['document_root'] = trim($m[1], '"\'');
        if (preg_match('/^ServerAlias\s+(.+)/i', $line, $m))       $current['aliases']       = preg_split('/\s+/', trim($m[1]));
    }
    return $hosts;
}

$vhosts = parseDevpanelConf($DEVPANEL_CONF);

// ── Validation ────────────────────────────────────────────────────────────
function checkPort(string $host, int $port): bool {
    $conn = @fsockopen($host, $port, $errstr, $errstr, 1);
    if ($conn !== false) { fclose($conn); return true; }
    return false;
}

$apacheOk = checkPort('127.0.0.1', 80);
$mysqlOk  = checkPort('127.0.0.1', 3306);

// Check devpanel.conf is enabled
$confEnabled = file_exists($SITES_ENABLED . '/devpanel.conf');
if (!$confEnabled) {
    $issues[] = ['level' => 'warning', 'msg' => 'devpanel.conf is not enabled in sites-enabled. Run: sudo a2ensite devpanel.conf'];
}

// Validate each vhost
$seenNames = [];
foreach ($vhosts as $vh) {
    $sn = $vh['server_name'];
    $dr = $vh['document_root'];

    if (isset($seenNames[$sn])) {
        $issues[] = ['level' => 'error', 'msg' => "Duplicate ServerName: $sn"];
    } else {
        $seenNames[$sn] = true;
    }

    if ($dr !== '' && !is_dir($dr)) {
        $issues[] = ['level' => 'warning', 'msg' => "DocumentRoot does not exist: $dr (for $sn)"];
    }

    // Check if entry in /etc/hosts
    $hostsContent = @file_get_contents('/etc/hosts') ?: '';
    if (!str_contains($hostsContent, $sn)) {
        $issues[] = ['level' => 'info', 'msg' => "$sn is not in /etc/hosts — add: 127.0.0.1 $sn"];
    }
}

// ── System info ───────────────────────────────────────────────────────────
$phpVersion = phpversion();
$phpModules = get_loaded_extensions();
sort($phpModules);
$hostname   = gethostname();

$memTotal  = 0;
$memFree   = 0;
if (is_readable('/proc/meminfo')) {
    foreach (file('/proc/meminfo') as $line) {
        if (str_starts_with($line, 'MemTotal:'))     sscanf($line, 'MemTotal: %d kB', $memTotal);
        if (str_starts_with($line, 'MemAvailable:')) sscanf($line, 'MemAvailable: %d kB', $memFree);
    }
}

$uptime = '';
if (is_readable('/proc/uptime')) {
    $sec = (int) file_get_contents('/proc/uptime');
    $uptime = sprintf('%dd %dh %dm', intdiv($sec, 86400), intdiv($sec % 86400, 3600), intdiv($sec % 3600, 60));
}
?>
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>DevPanel — Localhost</title>
<style>
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

:root {
    --bg:       #0e0e11;
    --surface:  #16161b;
    --card:     #1c1c23;
    --border:   #2a2a35;
    --text:     #e2e2e8;
    --muted:    #6b6b80;
    --teal:     #2dd4bf;
    --blue:     #60a5fa;
    --green:    #4ade80;
    --yellow:   #facc15;
    --red:      #f87171;
    --purple:   #c084fc;
    --orange:   #fb923c;
    --teal-bg:  rgba(45,212,191,.10);
    --blue-bg:  rgba(96,165,250,.10);
    --green-bg: rgba(74,222,128,.10);
    --red-bg:   rgba(248,113,113,.10);
    --yellow-bg:rgba(250,204,21,.10);
}

body { background: var(--bg); color: var(--text); font: 14px/1.6 'Inter', 'Segoe UI', system-ui, sans-serif; padding: 0; min-height: 100vh; }

.topbar {
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    padding: 14px 28px;
    display: flex; align-items: center; gap: 16px;
}
.logo { font-size: 16px; font-weight: 700; color: var(--teal); letter-spacing: -.3px; }
.logo span { color: var(--muted); font-weight: 400; }
.topbar-right { margin-left: auto; display: flex; gap: 10px; align-items: center; }

.status-dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; margin-right: 5px; }
.dot-ok  { background: var(--green); }
.dot-err { background: var(--red); }

.pill {
    display: inline-flex; align-items: center; gap: 5px;
    padding: 4px 10px; border-radius: 20px; font-size: 11px;
    border: 1px solid var(--border);
}
.pill-ok  { background: var(--green-bg);  color: var(--green);  border-color: rgba(74,222,128,.25); }
.pill-err { background: var(--red-bg);    color: var(--red);    border-color: rgba(248,113,113,.25); }
.pill-warn{ background: var(--yellow-bg); color: var(--yellow); border-color: rgba(250,204,21,.25); }
.pill-info{ background: var(--blue-bg);   color: var(--blue);   border-color: rgba(96,165,250,.25); }

.layout { display: grid; grid-template-columns: 260px 1fr; min-height: calc(100vh - 53px); }

.sidebar {
    background: var(--surface);
    border-right: 1px solid var(--border);
    padding: 24px 16px;
    display: flex; flex-direction: column; gap: 6px;
}
.sidebar-label { font-size: 10px; color: var(--muted); text-transform: uppercase; letter-spacing: .08em; padding: 8px 12px 4px; }

.content { padding: 28px 32px; }

.section-title { font-size: 17px; font-weight: 600; color: var(--text); margin-bottom: 4px; }
.section-sub   { font-size: 12px; color: var(--muted); margin-bottom: 20px; }

.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 14px; margin-bottom: 28px; }

.card {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 18px 20px;
}
.card-title { font-size: 13px; font-weight: 600; color: var(--text); margin-bottom: 6px; display: flex; align-items: center; gap: 8px; }
.card-sub   { font-size: 11px; color: var(--muted); margin-bottom: 14px; word-break: break-all; }
.card-divider { border: none; border-top: 1px solid var(--border); margin: 12px 0; }
.card-link  {
    display: inline-flex; align-items: center; gap: 6px;
    color: var(--teal); font-size: 12px; text-decoration: none; font-weight: 500;
    padding: 6px 12px; border-radius: 7px; border: 1px solid rgba(45,212,191,.25);
    background: var(--teal-bg); transition: opacity .15s;
}
.card-link:hover { opacity: .8; }

.vh-badge { font-size: 10px; padding: 2px 7px; border-radius: 5px; border: 1px solid var(--border); color: var(--muted); }

.stat-row { display: flex; justify-content: space-between; font-size: 12px; margin-bottom: 6px; }
.stat-val  { color: var(--teal); font-weight: 500; }

.issue-list { display: flex; flex-direction: column; gap: 8px; }
.issue {
    display: flex; gap: 12px; align-items: flex-start;
    padding: 10px 14px; border-radius: 8px; font-size: 12px;
}
.issue-error   { background: var(--red-bg);    border: 1px solid rgba(248,113,113,.2); }
.issue-warning { background: var(--yellow-bg); border: 1px solid rgba(250,204,21,.2); }
.issue-info    { background: var(--blue-bg);   border: 1px solid rgba(96,165,250,.2); }
.issue-icon { font-size: 13px; margin-top: 1px; flex-shrink: 0; }
.issue-msg  { color: var(--text); line-height: 1.5; }

.empty { color: var(--muted); font-size: 13px; padding: 20px 0; }

.modules { display: flex; flex-wrap: wrap; gap: 6px; }
.mod-badge {
    font-size: 10px; padding: 3px 8px; border-radius: 5px;
    background: var(--surface); border: 1px solid var(--border); color: var(--muted);
}

a { color: inherit; }
@media(max-width:700px) { .layout { grid-template-columns: 1fr; } .sidebar { display: none; } }
</style>
</head>
<body>

<div class="topbar">
    <div class="logo">DevPanel <span>Local</span></div>
    <div class="pill <?= $apacheOk ? 'pill-ok' : 'pill-err' ?>">
        <span class="status-dot <?= $apacheOk ? 'dot-ok' : 'dot-err' ?>"></span>
        Apache <?= $apacheOk ? 'running' : 'down' ?>
    </div>
    <div class="pill <?= $mysqlOk ? 'pill-ok' : 'pill-err' ?>">
        <span class="status-dot <?= $mysqlOk ? 'dot-ok' : 'dot-err' ?>"></span>
        MySQL <?= $mysqlOk ? 'running' : 'down' ?>
    </div>
    <div class="topbar-right" style="color:var(--muted);font-size:12px;">
        PHP <?= htmlspecialchars($phpVersion, ENT_QUOTES, 'UTF-8') ?>
        &nbsp;·&nbsp;
        <?= htmlspecialchars($hostname ?: 'localhost', ENT_QUOTES, 'UTF-8') ?>
        <?php if ($uptime): ?>
        &nbsp;·&nbsp; up <?= htmlspecialchars($uptime, ENT_QUOTES, 'UTF-8') ?>
        <?php endif; ?>
    </div>
</div>

<div class="layout">
    <!-- Sidebar nav -->
    <nav class="sidebar">
        <div class="sidebar-label">Quick Links</div>
        <a href="#projects" class="card-link" style="margin-bottom:2px;"> Projects</a>
        <a href="#vhosts"   class="card-link" style="margin-bottom:2px;"> Virtual Hosts</a>
        <a href="#system"   class="card-link" style="margin-bottom:2px;"> System</a>
        <?php if (!empty($issues)): ?>
        <a href="#issues"   class="card-link pill-warn" style="margin-bottom:2px;">
             Issues (<?= count($issues) ?>)
        </a>
        <?php endif; ?>

        <div class="sidebar-label" style="margin-top:12px;">Services</div>
        <div style="padding:10px 12px;">
            <div class="stat-row">
                <span>Apache 80</span>
                <span class="stat-val" style="color:<?= $apacheOk ? 'var(--green)' : 'var(--red)' ?>">
                    <?= $apacheOk ? '● OK' : '✕ Down' ?>
                </span>
            </div>
            <div class="stat-row">
                <span>MySQL 3306</span>
                <span class="stat-val" style="color:<?= $mysqlOk ? 'var(--green)' : 'var(--red)' ?>">
                    <?= $mysqlOk ? '● OK' : '✕ Down' ?>
                </span>
            </div>
        </div>

        <?php if ($memTotal > 0): ?>
        <div class="sidebar-label" style="margin-top:8px;">Memory</div>
        <div style="padding:10px 12px;">
            <?php
                $usedMb  = intdiv($memTotal - $memFree, 1024);
                $totalMb = intdiv($memTotal, 1024);
                $pct     = $totalMb > 0 ? round($usedMb / $totalMb * 100) : 0;
            ?>
            <div class="stat-row">
                <span>Used</span>
                <span class="stat-val"><?= $usedMb ?> / <?= $totalMb ?> MB</span>
            </div>
            <div style="background:var(--border);border-radius:4px;height:4px;margin-top:4px;">
                <div style="background:var(--teal);width:<?= $pct ?>%;height:4px;border-radius:4px;"></div>
            </div>
        </div>
        <?php endif; ?>
    </nav>

    <!-- Main content -->
    <main class="content">

        <!-- Projects -->
        <section id="projects">
            <div class="section-title">Projects</div>
            <div class="section-sub">
                Directories in <code><?= htmlspecialchars($PROJECTS_ROOT, ENT_QUOTES, 'UTF-8') ?></code> &nbsp;<span style="font-size:11px;color:var(--muted)">(separate from Apache webroot)</span>
            </div>
            <?php if (empty($projects)): ?>
                <p class="empty">No project directories found. Clone or create a project in ~/projects/</p>
            <?php else: ?>
            <div class="grid">
                <?php foreach ($projects as $p):
                    $hasIndex   = file_exists($PROJECTS_ROOT . '/' . $p . '/index.php')
                               || file_exists($PROJECTS_ROOT . '/' . $p . '/public/index.php');
                    $hasComposer = file_exists($PROJECTS_ROOT . '/' . $p . '/composer.json');
                    $hasGit     = is_dir($PROJECTS_ROOT . '/' . $p . '/.git');
                    // Try to find a vhost for this project
                    $matchedVhost = null;
                    foreach ($vhosts as $vh) {
                        if (str_contains($vh['server_name'], $p) || str_contains($vh['document_root'], $p)) {
                            $matchedVhost = $vh['server_name'];
                            break;
                        }
                    }
                ?>
                <div class="card">
                    <div class="card-title">
                        <?= htmlspecialchars($p, ENT_QUOTES, 'UTF-8') ?>
                    </div>
                    <div style="display:flex;gap:6px;flex-wrap:wrap;margin-bottom:12px;">
                        <?php if ($hasGit): ?>
                            <span class="vh-badge" style="color:var(--teal);border-color:rgba(45,212,191,.3)">git</span>
                        <?php endif; ?>
                        <?php if ($hasComposer): ?>
                            <span class="vh-badge" style="color:var(--purple);border-color:rgba(192,132,252,.3)">composer</span>
                        <?php endif; ?>
                        <?php if ($hasIndex): ?>
                            <span class="vh-badge" style="color:var(--green);border-color:rgba(74,222,128,.3)">index.php</span>
                        <?php endif; ?>
                    </div>
                    <hr class="card-divider">
                    <?php if ($matchedVhost): ?>
                        <a href="http://<?= htmlspecialchars($matchedVhost, ENT_QUOTES, 'UTF-8') ?>" class="card-link">
                            🌐 <?= htmlspecialchars($matchedVhost, ENT_QUOTES, 'UTF-8') ?>
                        </a>
                    <?php else: ?>
                        <span style="font-size:11px;color:var(--muted)">No vhost configured</span>
                    <?php endif; ?>
                </div>
                <?php endforeach; ?>
            </div>
            <?php endif; ?>
        </section>

        <!-- Virtual Hosts -->
        <section id="vhosts">
            <div class="section-title">Virtual Hosts</div>
            <div class="section-sub">
                Entries in <code><?= htmlspecialchars($DEVPANEL_CONF, ENT_QUOTES, 'UTF-8') ?></code>
                <?php if (!$confEnabled): ?>
                    &nbsp;<span class="pill pill-warn">not enabled</span>
                <?php else: ?>
                    &nbsp;<span class="pill pill-ok">enabled</span>
                <?php endif; ?>
            </div>
            <?php if (empty($vhosts)): ?>
                <p class="empty">No virtual hosts in devpanel.conf yet. Add one in DevPanel → VirtualHosts.</p>
            <?php else: ?>
            <div class="grid">
                <?php foreach ($vhosts as $vh): ?>
                <div class="card">
                    <div class="card-title">
                        <?= htmlspecialchars($vh['server_name'], ENT_QUOTES, 'UTF-8') ?>
                    </div>
                    <?php if ($vh['document_root']): ?>
                    <div class="card-sub">
                        <?= htmlspecialchars($vh['document_root'], ENT_QUOTES, 'UTF-8') ?>
                    </div>
                    <?php endif; ?>
                    <?php if (!empty($vh['aliases'])): ?>
                    <div style="font-size:11px;color:var(--muted);margin-bottom:12px;">
                        alias: <?= htmlspecialchars(implode(', ', $vh['aliases']), ENT_QUOTES, 'UTF-8') ?>
                    </div>
                    <?php endif; ?>
                    <hr class="card-divider">
                    <?php
                        $rootExists = $vh['document_root'] && is_dir($vh['document_root']);
                        $hostsLine  = str_contains(@file_get_contents('/etc/hosts') ?: '', $vh['server_name']);
                    ?>
                    <div style="display:flex;gap:8px;flex-wrap:wrap;align-items:center;">
                        <a href="http://<?= htmlspecialchars($vh['server_name'], ENT_QUOTES, 'UTF-8') ?>" class="card-link">
                            Open →
                        </a>
                        <?php if (!$rootExists): ?>
                            <span class="pill pill-warn" style="font-size:10px;">root missing</span>
                        <?php endif; ?>
                        <?php if (!$hostsLine): ?>
                            <span class="pill pill-warn" style="font-size:10px;">not in /etc/hosts</span>
                        <?php endif; ?>
                    </div>
                </div>
                <?php endforeach; ?>
            </div>
            <?php endif; ?>
        </section>

        <!-- Issues -->
        <?php if (!empty($issues)): ?>
        <section id="issues" style="margin-bottom:28px;">
            <div class="section-title">Issues</div>
            <div class="section-sub"><?= count($issues) ?> item<?= count($issues) !== 1 ? 's' : '' ?> need attention</div>
            <div class="issue-list">
                <?php foreach ($issues as $issue):
                    $cls  = match($issue['level']) { 'error' => 'issue-error', 'warning' => 'issue-warning', default => 'issue-info' };
                    $icon = match($issue['level']) { 'error' => '✕', 'warning' => '⚠', default => 'ℹ' };
                ?>
                <div class="issue <?= $cls ?>">
                    <span class="issue-icon"><?= $icon ?></span>
                    <span class="issue-msg"><?= htmlspecialchars($issue['msg'], ENT_QUOTES, 'UTF-8') ?></span>
                </div>
                <?php endforeach; ?>
            </div>
        </section>
        <?php endif; ?>

        <!-- System -->
        <section id="system">
            <div class="section-title">System</div>
            <div class="section-sub">PHP environment and loaded modules</div>
            <div class="grid">
                <div class="card">
                    <div class="card-title">PHP</div>
                    <div class="stat-row"><span>Version</span><span class="stat-val"><?= htmlspecialchars($phpVersion, ENT_QUOTES, 'UTF-8') ?></span></div>
                    <div class="stat-row"><span>SAPI</span><span class="stat-val"><?= htmlspecialchars(php_sapi_name(), ENT_QUOTES, 'UTF-8') ?></span></div>
                    <div class="stat-row"><span>Memory limit</span><span class="stat-val"><?= ini_get('memory_limit') ?></span></div>
                    <div class="stat-row"><span>Upload max</span><span class="stat-val"><?= ini_get('upload_max_filesize') ?></span></div>
                    <div class="stat-row"><span>Max execution</span><span class="stat-val"><?= ini_get('max_execution_time') ?>s</span></div>
                    <div class="stat-row"><span>Error display</span><span class="stat-val"><?= ini_get('display_errors') ? 'On' : 'Off' ?></span></div>
                </div>
                <div class="card">
                    <div class="card-title">Server</div>
                    <div class="stat-row"><span>Hostname</span><span class="stat-val"><?= htmlspecialchars($hostname ?: '-', ENT_QUOTES, 'UTF-8') ?></span></div>
                    <?php if ($uptime): ?><div class="stat-row"><span>Uptime</span><span class="stat-val"><?= htmlspecialchars($uptime, ENT_QUOTES, 'UTF-8') ?></span></div><?php endif; ?>
                    <?php if ($memTotal): ?>
                    <div class="stat-row"><span>RAM total</span><span class="stat-val"><?= intdiv($memTotal, 1024) ?> MB</span></div>
                    <div class="stat-row"><span>RAM free</span><span class="stat-val"><?= intdiv($memFree, 1024) ?> MB</span></div>
                    <?php endif; ?>
                    <div class="stat-row"><span>Apache port 80</span><span class="stat-val" style="color:<?= $apacheOk?'var(--green)':'var(--red)' ?>"><?= $apacheOk?'OK':'Down' ?></span></div>
                    <div class="stat-row"><span>MySQL port 3306</span><span class="stat-val" style="color:<?= $mysqlOk?'var(--green)':'var(--red)' ?>"><?= $mysqlOk?'OK':'Down' ?></span></div>
                </div>
            </div>

            <div class="card" style="margin-bottom:14px;">
                <div class="card-title">Loaded PHP Extensions</div>
                <div class="modules" style="margin-top:10px;">
                    <?php foreach ($phpModules as $mod): ?>
                    <span class="mod-badge"><?= htmlspecialchars($mod, ENT_QUOTES, 'UTF-8') ?></span>
                    <?php endforeach; ?>
                </div>
            </div>
        </section>

    </main>
</div>

<footer style="text-align:center;padding:16px;font-size:11px;color:var(--muted);border-top:1px solid var(--border);">
    DevPanel &nbsp;·&nbsp; PHP <?= htmlspecialchars($phpVersion, ENT_QUOTES, 'UTF-8') ?>
    &nbsp;·&nbsp; <?= htmlspecialchars($_SERVER['SERVER_SOFTWARE'] ?? 'Apache', ENT_QUOTES, 'UTF-8') ?>
</footer>

</body>
</html>
