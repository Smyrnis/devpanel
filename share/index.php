<?php

$DEVPANEL_CONF = '/etc/apache2/sites-available/devpanel.conf';
$SITES_ENABLED = '/etc/apache2/sites-enabled';

function e($value) {
    return htmlspecialchars($value, ENT_QUOTES | ENT_SUBSTITUTE, 'UTF-8');
}

function startsWithText($value, $prefix) {
    return substr($value, 0, strlen($prefix)) === $prefix;
}

function containsText($value, $needle) {
    return $needle === '' || strpos($value, $needle) !== false;
}

function divideInt($left, $right) {
    return $right === 0 ? 0 : (int) floor($left / $right);
}

function tomlValue($content, $key) {
    foreach (explode("\n", $content) as $line) {
        $line = trim($line);
        if ($line === '' || startsWithText($line, '#')) {
            continue;
        }
        $eq = strpos($line, '=');
        if ($eq === false) {
            continue;
        }
        $name = trim(substr($line, 0, $eq));
        if ($name !== $key) {
            continue;
        }
        $value = trim(substr($line, $eq + 1));
        if (strlen($value) >= 2 && $value[0] === '"' && substr($value, -1) === '"') {
            return substr($value, 1, -1);
        }
    }
    return '';
}

function configuredProjectsRoot() {
    $paths = glob('/home/*/.config/devpanel/config.toml');
    if ($paths === false) {
        return '';
    }

    foreach ($paths as $path) {
        if (!is_readable($path)) {
            continue;
        }
        $root = tomlValue((string) file_get_contents($path), 'projects_dir');
        if ($root !== '' && is_dir($root)) {
            return $root;
        }
    }
    return '';
}

function findProjectsRoot() {
    $configured = configuredProjectsRoot();
    if ($configured !== '') {
        return $configured;
    }

    $passwdLines = file('/etc/passwd') ?: [];
    foreach ($passwdLines as $line) {
        $parts = explode(':', trim($line));
        if (count($parts) < 7) {
            continue;
        }
        $uid = (int) $parts[2];
        $home = $parts[5];
        if ($uid < 1000 || $uid >= 65534) {
            continue;
        }
        if (is_dir("$home/projects")) {
            return "$home/projects";
        }
    }
    return '/var/www/html';
}

function parseDevpanelConf($path) {
    $hosts = [];
    if (!is_readable($path)) {
        return $hosts;
    }

    $content = (string) file_get_contents($path);
    $inBlock = false;
    $isHttps = false;
    $current = [];

    foreach (explode("\n", $content) as $raw) {
        $line = trim($raw);
        if (stripos($line, '<VirtualHost') === 0) {
            $inBlock = true;
            $isHttps = containsText($line, ':443');
            $current = [
                'server_name' => '',
                'document_root' => '',
                'aliases' => [],
                'php_version' => '',
                'https' => $isHttps,
            ];
            continue;
        }

        if (stripos($line, '</VirtualHost>') === 0 && $inBlock) {
            if ($current['server_name'] !== '') {
                $key = $current['server_name'];
                if (isset($hosts[$key])) {
                    $hosts[$key]['https'] = $hosts[$key]['https'] || $current['https'];
                    if ($hosts[$key]['php_version'] === '' && $current['php_version'] !== '') {
                        $hosts[$key]['php_version'] = $current['php_version'];
                    }
                } else {
                    $hosts[$key] = $current;
                }
            }
            $inBlock = false;
            continue;
        }

        if (!$inBlock) {
            continue;
        }

        if (preg_match('/^ServerName\s+(\S+)/i', $line, $m)) {
            $current['server_name'] = $m[1];
        }
        if (preg_match('/^DocumentRoot\s+(\S+)/i', $line, $m)) {
            $current['document_root'] = trim($m[1], '"\'');
        }
        if (preg_match('/^ServerAlias\s+(.+)/i', $line, $m)) {
            $current['aliases'] = preg_split('/\s+/', trim($m[1])) ?: [];
        }
        if (preg_match('/SetHandler\s+application\/x-httpd-php([0-9.]+)/i', $line, $m)) {
            $current['php_version'] = $m[1];
        }
        if (preg_match('/\/run\/php\/php([0-9.]+)-fpm\.sock/i', $line, $m)) {
            $current['php_version'] = $m[1];
        }
    }

    return array_values($hosts);
}

function checkPort($host, $port) {
    $conn = @fsockopen($host, $port, $errstr, $errstr, 0.35);
    if ($conn !== false) {
        fclose($conn);
        return true;
    }
    return false;
}

function projectLabelFromDocumentRoot($root, $projectsRoot) {
    $root = rtrim($root, '/');
    $projectsRoot = rtrim($projectsRoot, '/');
    if ($root === '') {
        return '';
    }

    if ($projectsRoot !== '' && startsWithText($root . '/', $projectsRoot . '/')) {
        $relative = substr($root, strlen($projectsRoot) + 1);
        $parts = explode('/', $relative);
        return $parts[0] !== '' ? $parts[0] : basename($root);
    }

    return basename($root);
}

function projectsFromVhosts($vhosts, $projectsRoot) {
    $projects = [];
    $seen = [];

    foreach ($vhosts as $host) {
        $root = isset($host['document_root']) ? $host['document_root'] : '';
        if ($root === '' || !is_dir($root)) {
            continue;
        }

        $label = projectLabelFromDocumentRoot($root, $projectsRoot);
        $key = $label . "\n" . $root;
        if (isset($seen[$key])) {
            continue;
        }

        $seen[$key] = true;
        $projects[] = [
            'name' => $label,
            'path' => $root,
            'server_name' => isset($host['server_name']) ? $host['server_name'] : '',
        ];
    }

    usort($projects, function ($a, $b) {
        return strcasecmp($a['name'], $b['name']);
    });
    return $projects;
}

function memoryStats() {
    $total = 0;
    $available = 0;
    if (is_readable('/proc/meminfo')) {
        foreach (file('/proc/meminfo') ?: [] as $line) {
            if (startsWithText($line, 'MemTotal:')) {
                sscanf($line, 'MemTotal: %d kB', $total);
            }
            if (startsWithText($line, 'MemAvailable:')) {
                sscanf($line, 'MemAvailable: %d kB', $available);
            }
        }
    }

    $usedMb = $total > 0 ? divideInt($total - $available, 1024) : 0;
    $totalMb = $total > 0 ? divideInt($total, 1024) : 0;
    $percent = $totalMb > 0 ? (int) round(($usedMb / $totalMb) * 100) : 0;

    return [$usedMb, $totalMb, $percent];
}

function uptimeLabel() {
    if (!is_readable('/proc/uptime')) {
        return 'unknown';
    }
    $seconds = (int) file_get_contents('/proc/uptime');
    return sprintf(
        '%dd %dh %dm',
        divideInt($seconds, 86400),
        divideInt($seconds % 86400, 3600),
        divideInt($seconds % 3600, 60)
    );
}

$projectsRoot = findProjectsRoot();
$vhosts = parseDevpanelConf($DEVPANEL_CONF);
$projects = projectsFromVhosts($vhosts, $projectsRoot);
$hostsContent = @file_get_contents('/etc/hosts') ?: '';

$apacheOk = checkPort('127.0.0.1', 80);
$mysqlOk = checkPort('127.0.0.1', 3306);
$confEnabled = file_exists($SITES_ENABLED . '/devpanel.conf');
$phpVersion = phpversion() ?: 'unknown';
$phpModules = get_loaded_extensions();
sort($phpModules);
$hostname = gethostname() ?: 'localhost';
list($memUsedMb, $memTotalMb, $memPct) = memoryStats();
$uptime = uptimeLabel();

$issues = [];
if (!$confEnabled) {
    $issues[] = ['warning', 'devpanel.conf is not enabled in Apache sites-enabled.'];
}
if (!$apacheOk) {
    $issues[] = ['error', 'Apache is not responding on port 80.'];
}

$seenNames = [];
foreach ($vhosts as $host) {
    $name = $host['server_name'];
    $root = $host['document_root'];
    if (isset($seenNames[$name])) {
        $issues[] = ['error', "Duplicate ServerName: $name"];
    }
    $seenNames[$name] = true;
    if ($root !== '' && !is_dir($root)) {
        $issues[] = ['warning', "DocumentRoot does not exist for $name: $root"];
    }
    if (!containsText($hostsContent, $name)) {
        $issues[] = ['info', "$name is not listed in /etc/hosts."];
    }
}

$services = [
    ['Apache', $apacheOk, '127.0.0.1:80'],
    ['MySQL', $mysqlOk, '127.0.0.1:3306'],
    ['DevPanel site', $confEnabled, 'devpanel.conf'],
];
?>
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>DevPanel Localhost</title>
<style>
*, *::before, *::after { box-sizing: border-box; }
:root {
    --bg: #f7f9fc;
    --panel: #ffffff;
    --panel-soft: #f8fafc;
    --border: #e1e7ef;
    --border-strong: #c9d3df;
    --text: #172033;
    --muted: #637083;
    --faint: #8b97a8;
    --green: #137b4c;
    --green-bg: #e9f7ef;
    --red: #bb3636;
    --red-bg: #fdeeee;
    --amber: #936012;
    --amber-bg: #fff6df;
    --blue: #1f5fbf;
    --blue-bg: #edf4ff;
    --shadow: 0 12px 28px rgba(15, 23, 42, .06);
    --radius: 8px;
    --mono: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    --font: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}
body {
    margin: 0;
    background:
        linear-gradient(180deg, #ffffff 0, var(--bg) 260px);
    color: var(--text);
    font-family: var(--font);
    font-size: 14px;
    line-height: 1.45;
}
a { color: inherit; text-decoration: none; }
code {
    font-family: var(--mono);
    font-size: 12px;
    color: #2e3b4f;
    background: #f3f6fa;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 2px 7px;
}
.page {
    max-width: 1160px;
    margin: 0 auto;
    padding: 30px 28px;
}
.topbar {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 18px;
    padding-bottom: 18px;
    border-bottom: 1px solid var(--border);
}
.brand {
    display: flex;
    align-items: center;
    min-width: 0;
}
.brand h1 {
    margin: 0;
    font-size: 23px;
    line-height: 1.1;
}
.brand p {
    margin: 4px 0 0;
    color: var(--muted);
    font-size: 13px;
}
.top-actions {
    margin-left: auto;
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
}
.button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-height: 36px;
    padding: 0 14px;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--panel);
    color: var(--text);
    font-size: 13px;
    font-weight: 650;
    box-shadow: 0 1px 0 rgba(15, 23, 42, .03);
    transition: border-color .15s ease, background .15s ease, transform .15s ease;
}
.button:hover {
    background: #f8fafc;
    border-color: var(--border-strong);
    transform: translateY(-1px);
}
.button.primary {
    background: #152033;
    border-color: #152033;
    color: #fff;
}
.button.primary:hover {
    background: #0f1828;
    border-color: #0f1828;
}
.hero {
    display: grid;
    grid-template-columns: minmax(0, 1.55fr) minmax(300px, .85fr);
    gap: 14px;
    margin-bottom: 16px;
}
.card {
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
}
.hero-main {
    padding: 22px;
}
.eyebrow {
    color: var(--blue);
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0;
}
.hero-title {
    max-width: 680px;
    margin: 7px 0 9px;
    font-size: 28px;
    letter-spacing: 0;
    line-height: 1.12;
}
.hero-copy {
    max-width: 680px;
    margin: 0;
    color: var(--muted);
}
.metric-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 8px;
    margin-top: 20px;
}
.metric {
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 12px;
    background: var(--panel-soft);
}
.metric span {
    display: block;
    color: var(--faint);
    font-size: 12px;
}
.metric strong {
    display: block;
    margin-top: 4px;
    font-size: 19px;
    line-height: 1.15;
}
.status-card {
    padding: 16px;
}
.status-list {
    display: grid;
    gap: 8px;
    margin-top: 12px;
}
.status-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 11px;
    border-radius: 8px;
    background: var(--panel-soft);
    border: 1px solid var(--border);
}
.status-row small {
    color: var(--muted);
    font-family: var(--mono);
}
.pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    min-height: 24px;
    padding: 0 9px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 700;
    white-space: nowrap;
}
.pill::before {
    content: "";
    width: 7px;
    height: 7px;
    border-radius: 999px;
    background: currentColor;
}
.ok { color: var(--green); background: var(--green-bg); }
.err { color: var(--red); background: var(--red-bg); }
.warn { color: var(--amber); background: var(--amber-bg); }
.info { color: var(--blue); background: var(--blue-bg); }
.section {
    margin-top: 16px;
}
.section-head {
    display: flex;
    align-items: end;
    justify-content: space-between;
    gap: 16px;
    margin: 0 0 9px;
}
.section-title {
    margin: 0;
    font-size: 17px;
    letter-spacing: 0;
}
.section-note {
    margin: 3px 0 0;
    color: var(--muted);
    font-size: 13px;
}
.grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(270px, 1fr));
    gap: 10px;
}
.site-card {
    padding: 14px;
    transition: border-color .15s ease, transform .15s ease;
}
.site-card:hover {
    border-color: var(--border-strong);
    transform: translateY(-1px);
}
.site-name {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin-bottom: 9px;
    font-weight: 700;
}
.path {
    display: block;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-bottom: 12px;
}
.chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 12px;
}
.chip {
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--panel-soft);
    color: var(--muted);
    font-size: 12px;
    font-weight: 650;
    padding: 3px 8px;
}
.open {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-height: 32px;
    padding: 0 12px;
    border-radius: 8px;
    background: var(--blue-bg);
    color: var(--blue);
    font-weight: 700;
    font-size: 13px;
    transition: background .15s ease, color .15s ease;
}
.open:hover {
    background: var(--blue);
    color: #fff;
}
.split {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(300px, .85fr);
    gap: 12px;
}
.list {
    display: grid;
    gap: 8px;
    padding: 13px;
}
.project-row,
.issue-row,
.module-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    background: var(--panel-soft);
    border-radius: 8px;
}
.issue-row {
    align-items: flex-start;
    justify-content: flex-start;
}
.issue-row strong {
    text-transform: capitalize;
    min-width: 58px;
}
.module-wrap {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    padding: 14px;
}
.module-row {
    display: inline-flex;
    width: auto;
    font-family: var(--mono);
    font-size: 12px;
}
.empty {
    padding: 20px;
    color: var(--muted);
}
.footer {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    margin: 28px 0 4px;
    color: var(--faint);
    font-size: 12px;
}
.bar {
    height: 8px;
    border-radius: 999px;
    background: #e8edf4;
    overflow: hidden;
    margin-top: 8px;
}
.bar span {
    display: block;
    height: 100%;
    background: #1f5fbf;
    width: <?= max(0, min(100, $memPct)) ?>%;
}
@media (max-width: 860px) {
    .page { padding: 18px; }
    .topbar, .section-head { align-items: flex-start; flex-direction: column; }
    .top-actions { margin-left: 0; }
    .hero, .split { grid-template-columns: 1fr; }
    .metric-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
}
@media (max-width: 520px) {
    .metric-grid { grid-template-columns: 1fr; }
    .hero-title { font-size: 25px; }
}
</style>
</head>
<body>
<div class="page">
    <header class="topbar">
        <div class="brand">
            <div>
                <h1>DevPanel</h1>
                <p>Local workspace on <?= e($hostname) ?></p>
            </div>
        </div>
        <div class="top-actions">
            <a class="button primary" href="http://localhost">Localhost</a>
            <a class="button" href="http://localhost/phpmyadmin">phpMyAdmin</a>
            <a class="button" href="/index.html">Apache default</a>
        </div>
    </header>

    <section class="hero">
        <div class="card hero-main">
            <div class="eyebrow">Workspace overview</div>
            <h2 class="hero-title">Local sites, services, and PHP status in one place.</h2>
            <p class="hero-copy">
                This page reads DevPanel's Apache virtual hosts, linked project roots,
                PHP runtime, and service ports so you can open sites and spot routing issues quickly.
            </p>
            <div class="metric-grid">
                <div class="metric"><span>Virtual hosts</span><strong><?= count($vhosts) ?></strong></div>
                <div class="metric"><span>Projects</span><strong><?= count($projects) ?></strong></div>
                <div class="metric"><span>PHP</span><strong><?= e($phpVersion) ?></strong></div>
                <div class="metric"><span>Uptime</span><strong><?= e($uptime) ?></strong></div>
            </div>
        </div>
        <aside class="card status-card">
            <div class="section-head">
                <div>
                    <h2 class="section-title">Services</h2>
                    <p class="section-note">Current local port and site state.</p>
                </div>
            </div>
            <div class="status-list">
                <?php foreach ($services as $service): ?>
                    <?php $name = $service[0]; $ok = $service[1]; $detail = $service[2]; ?>
                    <div class="status-row">
                        <div>
                            <strong><?= e($name) ?></strong><br>
                            <small><?= e($detail) ?></small>
                        </div>
                        <span class="pill <?= $ok ? 'ok' : 'err' ?>"><?= $ok ? 'OK' : 'Check' ?></span>
                    </div>
                <?php endforeach; ?>
                <div class="status-row">
                    <div>
                        <strong>Memory</strong><br>
                        <small><?= $memUsedMb ?> MB / <?= $memTotalMb ?> MB</small>
                        <div class="bar"><span></span></div>
                    </div>
                    <span class="pill info"><?= $memPct ?>%</span>
                </div>
            </div>
        </aside>
    </section>

    <section class="section">
        <div class="section-head">
            <div>
                <h2 class="section-title">Virtual Hosts</h2>
                <p class="section-note"><code><?= e($DEVPANEL_CONF) ?></code></p>
            </div>
            <span class="pill <?= $confEnabled ? 'ok' : 'warn' ?>"><?= $confEnabled ? 'enabled' : 'not enabled' ?></span>
        </div>
        <?php if (empty($vhosts)): ?>
            <div class="card empty">No DevPanel virtual hosts were found.</div>
        <?php else: ?>
            <div class="grid">
                <?php foreach ($vhosts as $host): ?>
                    <article class="card site-card">
                        <div class="site-name">
                            <span><?= e($host['server_name']) ?></span>
                            <span class="pill <?= $host['https'] ? 'ok' : 'info' ?>"><?= $host['https'] ? 'HTTPS' : 'HTTP' ?></span>
                        </div>
                        <code class="path"><?= e($host['document_root'] ?: 'No DocumentRoot') ?></code>
                        <div class="chips">
                            <span class="chip">PHP <?= e($host['php_version'] ?: $phpVersion) ?></span>
                            <?php if (!empty($host['aliases'])): ?>
                                <span class="chip"><?= count($host['aliases']) ?> alias<?= count($host['aliases']) === 1 ? '' : 'es' ?></span>
                            <?php endif; ?>
                            <span class="chip"><?= containsText($hostsContent, $host['server_name']) ? 'hosts OK' : 'hosts missing' ?></span>
                        </div>
                        <a class="open" href="http://<?= e($host['server_name']) ?>">Open site</a>
                    </article>
                <?php endforeach; ?>
            </div>
        <?php endif; ?>
    </section>

    <section class="section split">
        <div class="card">
            <div class="list">
                <div>
                    <h2 class="section-title">Projects</h2>
                    <p class="section-note">Existing DocumentRoot entries from <code><?= e($DEVPANEL_CONF) ?></code></p>
                </div>
                <?php if (empty($projects)): ?>
                    <div class="empty">No existing project DocumentRoot entries were found.</div>
                <?php else: ?>
                    <?php foreach (array_slice($projects, 0, 12) as $project): ?>
                        <div class="project-row">
                            <strong><?= e($project['name']) ?></strong>
                            <code><?= e($project['path']) ?></code>
                            <?php if ($project['server_name'] !== ''): ?>
                                <span><?= e($project['server_name']) ?></span>
                            <?php endif; ?>
                        </div>
                    <?php endforeach; ?>
                    <?php if (count($projects) > 12): ?>
                        <div class="project-row"><strong><?= count($projects) - 12 ?> more</strong><span>Hidden for scanability</span></div>
                    <?php endif; ?>
                <?php endif; ?>
            </div>
        </div>

        <div class="card">
            <div class="list">
                <div>
                    <h2 class="section-title">Checks</h2>
                    <p class="section-note">Warnings that can affect local routing.</p>
                </div>
                <?php if (empty($issues)): ?>
                    <div class="issue-row"><span class="pill ok">OK</span><span>No routing or setup issues detected.</span></div>
                <?php else: ?>
                    <?php foreach ($issues as $issue): ?>
                        <?php $level = $issue[0]; $message = $issue[1]; ?>
                        <div class="issue-row">
                            <strong class="<?= $level === 'error' ? 'err' : ($level === 'warning' ? 'warn' : 'info') ?>"><?= e($level) ?></strong>
                            <span><?= e($message) ?></span>
                        </div>
                    <?php endforeach; ?>
                <?php endif; ?>
            </div>
        </div>
    </section>

    <section class="section">
        <div class="section-head">
            <div>
                <h2 class="section-title">PHP Runtime</h2>
                <p class="section-note">Loaded extensions for the PHP process serving this page.</p>
            </div>
            <span class="pill info"><?= count($phpModules) ?> modules</span>
        </div>
        <div class="card module-wrap">
            <?php foreach ($phpModules as $module): ?>
                <span class="module-row"><?= e($module) ?></span>
            <?php endforeach; ?>
        </div>
    </section>

    <footer class="footer">
        <span>DevPanel localhost page</span>
        <span>/</span>
        <span>Apache <?= $apacheOk ? 'online' : 'offline' ?></span>
        <span>/</span>
        <span>PHP <?= e($phpVersion) ?></span>
        <span>/</span>
        <span><?= e(date('Y-m-d H:i:s')) ?></span>
    </footer>
</div>
</body>
</html>
