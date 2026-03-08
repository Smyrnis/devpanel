<?php
declare(strict_types=1);
/**
 * DevPanel Welcome Page
 * Served from /var/www/html — the standard Apache webroot.
 * Reads virtual hosts from /etc/apache2/sites-available/devpanel.conf
 */

$DEVPANEL_CONF = '/etc/apache2/sites-available/devpanel.conf';
$SITES_ENABLED = '/etc/apache2/sites-enabled';

function findProjectsRoot(): string {
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
        if (is_dir("$home/projects")) return "$home/projects";
    }
    return '/var/www/html';
}

$PROJECTS_ROOT = findProjectsRoot();

$projects = [];
if (is_dir($PROJECTS_ROOT) && is_readable($PROJECTS_ROOT)) {
    foreach (scandir($PROJECTS_ROOT) as $entry) {
        if ($entry === '.' || $entry === '..' || str_starts_with($entry, '.')) continue;
        $full = $PROJECTS_ROOT . '/' . $entry;
        if (is_dir($full)) $projects[] = $entry;
    }
}

$vhosts = [];
$issues = [];

function parseDevpanelConf(string $path): array {
    $hosts = [];
    if (!is_readable($path)) return $hosts;
    $content = file_get_contents($path);
    $inBlock = false;
    $current = [];
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
        if (preg_match('/^ServerName\s+(\S+)/i', $line, $m))  $current['server_name']   = $m[1];
        if (preg_match('/^DocumentRoot\s+(\S+)/i', $line, $m)) $current['document_root'] = trim($m[1], '"\'');
        if (preg_match('/^ServerAlias\s+(.+)/i', $line, $m))   $current['aliases']       = preg_split('/\s+/', trim($m[1]));
    }
    return $hosts;
}

$vhosts = parseDevpanelConf($DEVPANEL_CONF);

function checkPort(string $host, int $port): bool {
    $conn = @fsockopen($host, $port, $errstr, $errstr, 1);
    if ($conn !== false) { fclose($conn); return true; }
    return false;
}

$apacheOk   = checkPort('127.0.0.1', 80);
$mysqlOk    = checkPort('127.0.0.1', 3306);
$confEnabled = file_exists($SITES_ENABLED . '/devpanel.conf');

if (!$confEnabled) {
    $issues[] = ['level' => 'warning', 'msg' => 'devpanel.conf is not enabled in sites-enabled. Run: sudo a2ensite devpanel.conf'];
}

$seenNames = [];
$hostsContent = @file_get_contents('/etc/hosts') ?: '';
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
    if (!str_contains($hostsContent, $sn)) {
        $issues[] = ['level' => 'info', 'msg' => "$sn is not in /etc/hosts — add: 127.0.0.1 $sn"];
    }
}

$phpVersion = phpversion();
$phpModules = get_loaded_extensions();
sort($phpModules);
$hostname   = gethostname();

$memTotal = 0;
$memFree  = 0;
if (is_readable('/proc/meminfo')) {
    foreach (file('/proc/meminfo') as $line) {
        if (str_starts_with($line, 'MemTotal:'))     sscanf($line, 'MemTotal: %d kB', $memTotal);
        if (str_starts_with($line, 'MemAvailable:')) sscanf($line, 'MemAvailable: %d kB', $memFree);
    }
}

$uptime = '';
if (is_readable('/proc/uptime')) {
    $sec    = (int) file_get_contents('/proc/uptime');
    $uptime = sprintf('%dd %dh %dm', intdiv($sec, 86400), intdiv($sec % 86400, 3600), intdiv($sec % 3600, 60));
}

$memUsedMb  = $memTotal > 0 ? intdiv($memTotal - $memFree, 1024) : 0;
$memTotalMb = intdiv($memTotal, 1024);
$memPct     = $memTotalMb > 0 ? round($memUsedMb / $memTotalMb * 100) : 0;
?>
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>DevPanel</title>
<link rel="icon" type="image/png" href="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAYAAABzenr0AAAEtUlEQVR4nMWXS4sdVRDHf1WnX7fv5B2NJguJ6EJFAyYi4pdQiRjBx86NX8GtKPgJdKeCC4ludZGtCyWiMT4IoigEFYZJJsmdvt33PMpF3xln5t6ZhMwFCw403adP/U/V/1/nlAACGEBd168DbwBPAMPpt0WYAWvAD8AHTdN8OH0vAihQ1nX9kYicNbMF+ZxvIoKZnW+a5jWgE0Dquj4vIi+klEKWZZplmYosavO9mRkhhBRCSKqamdnnTdOcleFw+ArwcUrJF0WRZ1lGSsnMLCKoILoI5yKCqhJjpOs6r6o58KrUdf0NcMY5Z0VRaIwRESFzGWaGsZiUpJQwM5xzTCaTFGMU4KLUdT0GqrIsoSeLlHU1Kh/c93ZYbp9ON/xzZpboubInm0wmrHOs6zqAVoEKpuTAkhOH1tkvf37/67t2at97MUainzNiJKV0xwNAVTfSMeVYlW1GKIgzMdIt/+SJhx/4hJ/XHnF1DjKVo0GyfjG8YT7tWajZzBsVrE2Ok4OX3TOH0HFgHa4IVEWFDnOaC38zvvAPUmeQ7p4nMwDEIOXGPb+X4f72gIYQFPqQhRC4cuUK3nswQyq3J+dzAUAvmyIvsqXBsHc2zZmfTBivNQDkWb4QhcxltplRFAXDpSXq4ZDhdBw6fJhz585x/PhxfPAsoljNRGBdqysrK1sYPJlMNrS8urqKc45FlO1ZDkx3FWMkhLAB4OrVq3Rd1xepLEN1z2VhPoCUEnmes7q6yvXr1zfeqyqDwQDgrnY+PYRuD8DM8N7jnJvJsW1m/J2m3wxE8L6P5vY156oghmghxg5QmUNUm/o3s0wwDImI2DxMU6WYmGWIzkyZB8DbfncN7IBhPlpq1v0lwImKqpopUdbi0MJAYnl4JBZdTNJzZpObzDkQMbGQXLd8GLNiZwAGZNIlZwNdyr5UbyeGVj7Vh80QUdqu7cmoiitZNqtJJscgl7KAqiz7XIuAGaO1MZYiSYtbqkUroS0QWQ/i3AjU2VL+9fW/rj3/4qdvntr32ehbJyoxRfIsl+9+vMSlny5RVlXCRAiN6NFH4/jZt9zjv73D6cdO0k4CIsLEB7648JWtNWNRNZjDHKnr2gCqqupJlkmT9kuj91bv6zg9tNSWLwlCtMTKzWuICk4diODEltP4IKE4eNSWjslwfJW6VATh1thzq5lQ5P0eTfNR7peThHa/qFrXdWJm2wCYgTCxfXrD4ABmCZW1dbQqQq8kM1SiNGGQQi2pONjgW0c+IBnCNAOqgvXaM7EYew6kQkTouo75AHpLCC2gbLmSbdGxkMgRA9T3ed2mc9uYCWYZU0VtBjBXhoBi1HOcbkvgxuo5uxWnXT4tpp7uwf53ABnQAtXmOn1Hx6zRl1mV/lIyW+Rmf9nkY/rcZsBl4EwIwYqi0JQSdV1vXCB3MlGBXLA2IZViberjuUu+R6MRqor3PtGT5/KWxqQsy3zeIfSfVzCfyO4bUJ0+Qv7gEu3Fa1gXiCsT/B8jpNAdQZjZbGPCDq3Zhs/tJ6IZooJUDnekJK0F0s3+jmjR5s5ftxhj8t5vbc3YoTndlQc2PZqj9ZXCSR+e3X7ZtO5Mc8rdtufrX29/P9mxPf8XYCbQoKWbgw4AAAAASUVORK5CYII=">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=DM+Mono:ital,wght@0,300;0,400;0,500;1,400&family=DM+Sans:ital,opsz,wght@0,9..40,300;0,9..40,400;0,9..40,500;0,9..40,600;1,9..40,300&display=swap" rel="stylesheet">
<style>
/* ── Reset ── */
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

/* ── Tokens ── */
:root {
    --bg:          #0a0a0a;
    --surface:     #111111;
    --surface-2:   #161616;
    --surface-3:   #1c1c1c;
    --border:      rgba(255,255,255,.07);
    --border-mid:  rgba(255,255,255,.11);
    --text:        #f0f0f0;
    --text-2:      #a0a0a0;
    --text-3:      #555;
    --green:       #30d158;
    --green-dim:   rgba(48,209,88,.12);
    --green-ring:  rgba(48,209,88,.28);
    --red:         #ff453a;
    --red-dim:     rgba(255,69,58,.10);
    --yellow:      #ffd60a;
    --yellow-dim:  rgba(255,214,10,.10);
    --blue:        #0a84ff;
    --blue-dim:    rgba(10,132,255,.10);
    --radius-sm:   6px;
    --radius:      12px;
    --radius-lg:   16px;
    --font:        'DM Sans', system-ui, sans-serif;
    --mono:        'DM Mono', 'SF Mono', monospace;
    --ease:        cubic-bezier(.25,.46,.45,.94);
}

/* ── Base ── */
html { scroll-behavior: smooth; }
body {
    background: var(--bg);
    color: var(--text);
    font-family: var(--font);
    font-size: 14px;
    line-height: 1.5;
    min-height: 100vh;
    -webkit-font-smoothing: antialiased;
}
a { color: inherit; text-decoration: none; }
code {
    font-family: var(--mono);
    font-size: .85em;
    color: var(--text-2);
    background: var(--surface-3);
    padding: 1px 6px;
    border-radius: 4px;
    border: 1px solid var(--border);
}

/* ── Layout ── */
.shell {
    display: grid;
    grid-template-columns: 220px 1fr;
    grid-template-rows: 52px 1fr auto;
    grid-template-areas:
        "topbar topbar"
        "sidebar main"
        "footer footer";
    min-height: 100vh;
}

/* ── Topbar ── */
.topbar {
    grid-area: topbar;
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 0 24px;
    background: rgba(10,10,10,.85);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    z-index: 100;
}

.logo {
    font-size: 15px;
    font-weight: 600;
    color: var(--text);
    letter-spacing: -.3px;
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
}
.logo-mark {
    width: 22px;
    height: 22px;
    background: var(--green);
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
}
.logo-mark svg { width: 12px; height: 12px; }
.logo-sub { color: var(--text-3); font-weight: 400; font-size: 13px; }

.topbar-sep { width: 1px; height: 18px; background: var(--border-mid); flex-shrink: 0; }

.service-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 20px;
    font-size: 12px;
    font-weight: 500;
    border: 1px solid transparent;
    transition: opacity .15s;
}
.badge-ok   { background: var(--green-dim); color: var(--green); border-color: var(--green-ring); }
.badge-err  { background: var(--red-dim);   color: var(--red);   border-color: rgba(255,69,58,.25); }
.badge-warn { background: var(--yellow-dim);color: var(--yellow);border-color: rgba(255,214,10,.25); }
.badge-info { background: var(--blue-dim);  color: var(--blue);  border-color: rgba(10,132,255,.22); }

.dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
}
.dot-ok  { background: var(--green); box-shadow: 0 0 6px var(--green); }
.dot-err { background: var(--red);   box-shadow: 0 0 6px var(--red); }

.topbar-meta {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 16px;
    color: var(--text-3);
    font-size: 12px;
    font-family: var(--mono);
}
.topbar-meta span { color: var(--text-2); }

/* ── Sidebar ── */
.sidebar {
    grid-area: sidebar;
    background: var(--surface);
    border-right: 1px solid var(--border);
    padding: 20px 12px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    position: sticky;
    top: 52px;
    height: calc(100vh - 52px);
    overflow-y: auto;
}

.nav-label {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-3);
    text-transform: uppercase;
    letter-spacing: .1em;
    padding: 10px 10px 4px;
    margin-top: 4px;
}
.nav-label:first-child { margin-top: 0; }

.nav-item {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 7px 10px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    font-weight: 400;
    color: var(--text-2);
    transition: background .12s var(--ease), color .12s var(--ease);
    cursor: pointer;
}
.nav-item:hover { background: var(--surface-3); color: var(--text); }
.nav-item svg { width: 14px; height: 14px; flex-shrink: 0; opacity: .7; }
.nav-item.warn { color: var(--yellow); }
.nav-item.warn svg { opacity: 1; }

.nav-count {
    margin-left: auto;
    font-size: 10px;
    font-weight: 600;
    background: rgba(255,214,10,.15);
    color: var(--yellow);
    padding: 1px 6px;
    border-radius: 10px;
}

.sidebar-divider { height: 1px; background: var(--border); margin: 8px 10px; }

/* Service status in sidebar */
.service-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 5px 10px;
    font-size: 12px;
    color: var(--text-2);
}
.service-name { font-family: var(--mono); font-size: 11px; }
.service-status-ok  { color: var(--green); font-weight: 500; font-size: 11px; }
.service-status-err { color: var(--red);   font-weight: 500; font-size: 11px; }

/* Memory bar */
.mem-block { padding: 4px 10px 10px; }
.mem-label { display: flex; justify-content: space-between; font-size: 11px; color: var(--text-2); margin-bottom: 6px; }
.mem-label code { background: none; border: none; padding: 0; font-size: 11px; color: var(--green); }
.mem-track {
    height: 3px;
    background: var(--surface-3);
    border-radius: 2px;
    overflow: hidden;
}
.mem-fill {
    height: 3px;
    background: var(--green);
    border-radius: 2px;
    transition: width .4s var(--ease);
}

/* ── Main content ── */
.main {
    grid-area: main;
    padding: 32px 36px;
    max-width: 1100px;
}

/* Section headers */
.sec-head {
    display: flex;
    align-items: baseline;
    gap: 10px;
    margin-bottom: 6px;
}
.sec-title {
    font-size: 17px;
    font-weight: 600;
    color: var(--text);
    letter-spacing: -.3px;
}
.sec-count {
    font-size: 12px;
    color: var(--text-3);
    font-family: var(--mono);
}
.sec-sub {
    font-size: 12px;
    color: var(--text-3);
    margin-bottom: 18px;
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
}

section { margin-bottom: 40px; }

/* ── Cards grid ── */
.grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 12px;
}

.card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 18px;
    transition: border-color .15s var(--ease), background .15s var(--ease);
}
.card:hover { border-color: var(--border-mid); background: var(--surface-2); }

.card-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
    letter-spacing: -.15px;
    margin-bottom: 8px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}
.card-path {
    font-family: var(--mono);
    font-size: 10.5px;
    color: var(--text-3);
    margin-bottom: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

/* Tag chips */
.chips { display: flex; flex-wrap: wrap; gap: 5px; margin-bottom: 14px; }
.chip {
    font-size: 10px;
    font-weight: 500;
    padding: 2px 7px;
    border-radius: 4px;
    border: 1px solid transparent;
    font-family: var(--mono);
}
.chip-green  { background: var(--green-dim);  color: var(--green);  border-color: var(--green-ring); }
.chip-blue   { background: var(--blue-dim);   color: var(--blue);   border-color: rgba(10,132,255,.22); }
.chip-purple { background: rgba(191,90,242,.10); color: #bf5af2; border-color: rgba(191,90,242,.22); }
.chip-muted  { background: var(--surface-3);  color: var(--text-3); border-color: var(--border); }

.card-divider { height: 1px; background: var(--border); margin: 12px 0; }

/* Open link */
.open-link {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    font-weight: 500;
    color: var(--green);
    padding: 5px 11px;
    border-radius: var(--radius-sm);
    background: var(--green-dim);
    border: 1px solid var(--green-ring);
    transition: opacity .15s;
}
.open-link:hover { opacity: .75; }
.open-link svg { width: 11px; height: 11px; }

.no-vhost { font-size: 11px; color: var(--text-3); font-family: var(--mono); }

/* ── Stats rows (for System section) ── */
.stat-group { display: flex; flex-direction: column; gap: 0; }
.stat-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
    font-size: 13px;
    color: var(--text-2);
}
.stat-row:last-child { border-bottom: none; }
.stat-key { font-size: 12px; }
.stat-val {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--text);
    font-weight: 500;
}
.stat-ok  { color: var(--green) !important; }
.stat-err { color: var(--red) !important; }

/* ── Issues ── */
.issue-list { display: flex; flex-direction: column; gap: 8px; }
.issue {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 12px 16px;
    border-radius: var(--radius);
    font-size: 13px;
    border: 1px solid transparent;
    line-height: 1.55;
}
.issue-error   { background: var(--red-dim);    border-color: rgba(255,69,58,.2); }
.issue-warning { background: var(--yellow-dim); border-color: rgba(255,214,10,.2); }
.issue-info    { background: var(--blue-dim);   border-color: rgba(10,132,255,.2); }
.issue-icon {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
    margin-top: 1px;
}
.issue-msg { color: var(--text); }

/* ── PHP Modules ── */
.modules { display: flex; flex-wrap: wrap; gap: 5px; margin-top: 12px; }
.mod {
    font-family: var(--mono);
    font-size: 10.5px;
    color: var(--text-3);
    background: var(--surface-2);
    border: 1px solid var(--border);
    padding: 2px 7px;
    border-radius: 4px;
}

/* ── Empty states ── */
.empty {
    font-size: 13px;
    color: var(--text-3);
    padding: 24px 0;
}

/* ── Inline pill for conf status ── */
.inline-pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    padding: 2px 8px;
    border-radius: 10px;
    border: 1px solid transparent;
    font-weight: 500;
}

/* ── Footer ── */
.footer {
    grid-area: footer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 14px 24px;
    border-top: 1px solid var(--border);
    font-size: 11px;
    color: var(--text-3);
    font-family: var(--mono);
}
.footer-sep { color: var(--border-mid); }

/* ── Responsive ── */
@media (max-width: 720px) {
    .shell { grid-template-columns: 1fr; grid-template-areas: "topbar" "main" "footer"; }
    .sidebar { display: none; }
    .main { padding: 20px 18px; }
}

/* ── Animations ── */
@keyframes fadeUp {
    from { opacity: 0; transform: translateY(10px); }
    to   { opacity: 1; transform: translateY(0); }
}
section {
    animation: fadeUp .35s var(--ease) both;
}
section:nth-child(1) { animation-delay: .05s; }
section:nth-child(2) { animation-delay: .12s; }
section:nth-child(3) { animation-delay: .19s; }
section:nth-child(4) { animation-delay: .26s; }
</style>
</head>
<body>
<div class="shell">

<!-- ── Topbar ── -->
<header class="topbar">
    <div class="logo">
        <div class="logo-mark">
            <svg viewBox="0 0 12 12" fill="none" xmlns="http://www.w3.org/2000/svg">
                <rect x="1" y="1" width="4" height="4" rx="1" fill="#0a0a0a"/>
                <rect x="7" y="1" width="4" height="4" rx="1" fill="#0a0a0a"/>
                <rect x="1" y="7" width="4" height="4" rx="1" fill="#0a0a0a"/>
                <rect x="7" y="7" width="4" height="4" rx="1" fill="#0a0a0a" opacity=".4"/>
            </svg>
        </div>
        DevPanel <span class="logo-sub">/ Local</span>
    </div>

    <div class="topbar-sep"></div>

    <div class="service-badge <?= $apacheOk ? 'badge-ok' : 'badge-err' ?>">
        <span class="dot <?= $apacheOk ? 'dot-ok' : 'dot-err' ?>"></span>
        Apache
    </div>

    <div class="service-badge <?= $mysqlOk ? 'badge-ok' : 'badge-err' ?>">
        <span class="dot <?= $mysqlOk ? 'dot-ok' : 'dot-err' ?>"></span>
        MySQL
    </div>

    <?php if (!empty($issues)): ?>
    <div class="service-badge badge-warn">
        <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor"><path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 10.5a.75.75 0 1 1 0-1.5.75.75 0 0 1 0 1.5zm.75-3.5a.75.75 0 0 1-1.5 0v-3a.75.75 0 0 1 1.5 0v3z"/></svg>
        <?= count($issues) ?> issue<?= count($issues) !== 1 ? 's' : '' ?>
    </div>
    <?php endif; ?>

    <div class="topbar-meta">
        <span>php <?= htmlspecialchars($phpVersion, ENT_QUOTES, 'UTF-8') ?></span>
        <span class="topbar-sep" style="height:12px;"></span>
        <span><?= htmlspecialchars($hostname ?: 'localhost', ENT_QUOTES, 'UTF-8') ?></span>
        <?php if ($uptime): ?>
        <span class="topbar-sep" style="height:12px;"></span>
        <span><?= htmlspecialchars($uptime, ENT_QUOTES, 'UTF-8') ?></span>
        <?php endif; ?>
    </div>
</header>

<!-- ── Sidebar ── -->
<nav class="sidebar">
    <div class="nav-label">Navigate</div>
    <a href="#projects" class="nav-item">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="3" width="12" height="10" rx="1.5"/><path d="M5 3V2m6 1V2"/></svg>
        Projects
        <?php if (!empty($projects)): ?><span class="nav-count" style="background:var(--green-dim);color:var(--green);"><?= count($projects) ?></span><?php endif; ?>
    </a>
    <a href="#vhosts" class="nav-item">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="6"/><path d="M8 2a9 9 0 0 1 0 12M8 2a9 9 0 0 0 0 12M2 8h12"/></svg>
        Virtual Hosts
        <?php if (!empty($vhosts)): ?><span class="nav-count" style="background:var(--blue-dim);color:var(--blue);"><?= count($vhosts) ?></span><?php endif; ?>
    </a>
    <a href="#system" class="nav-item">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="12" height="12" rx="1.5"/><path d="M5 8h6M8 5v6"/></svg>
        System
    </a>
    <?php if (!empty($issues)): ?>
    <a href="#issues" class="nav-item warn">
        <svg viewBox="0 0 16 16" fill="currentColor"><path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 10.5a.75.75 0 1 1 0-1.5.75.75 0 0 1 0 1.5zm.75-3.5a.75.75 0 0 1-1.5 0v-3a.75.75 0 0 1 1.5 0v3z"/></svg>
        Issues
        <span class="nav-count"><?= count($issues) ?></span>
    </a>
    <?php endif; ?>

    <div class="sidebar-divider"></div>
    <div class="nav-label">Services</div>

    <div class="service-row">
        <span class="service-name">apache :80</span>
        <span class="<?= $apacheOk ? 'service-status-ok' : 'service-status-err' ?>"><?= $apacheOk ? 'running' : 'down' ?></span>
    </div>
    <div class="service-row">
        <span class="service-name">mysql :3306</span>
        <span class="<?= $mysqlOk ? 'service-status-ok' : 'service-status-err' ?>"><?= $mysqlOk ? 'running' : 'down' ?></span>
    </div>

    <?php if ($memTotalMb > 0): ?>
    <div class="sidebar-divider"></div>
    <div class="nav-label">Memory</div>
    <div class="mem-block">
        <div class="mem-label">
            <span><?= $memUsedMb ?> MB used</span>
            <code><?= $memPct ?>%</code>
        </div>
        <div class="mem-track">
            <div class="mem-fill" style="width:<?= $memPct ?>%"></div>
        </div>
        <div class="mem-label" style="margin-top:5px;margin-bottom:0;">
            <span></span>
            <span><?= $memTotalMb ?> MB total</span>
        </div>
    </div>
    <?php endif; ?>
</nav>

<!-- ── Main ── -->
<main class="main">

    <!-- Projects -->
    <section id="projects">
        <div class="sec-head">
            <div class="sec-title">Projects</div>
            <?php if (!empty($projects)): ?>
            <div class="sec-count"><?= count($projects) ?> found</div>
            <?php endif; ?>
        </div>
        <div class="sec-sub">
            Scanning <code><?= htmlspecialchars($PROJECTS_ROOT, ENT_QUOTES, 'UTF-8') ?></code>
        </div>

        <?php if (empty($projects)): ?>
            <p class="empty">No project directories found. Clone or create a project in ~/projects/</p>
        <?php else: ?>
        <div class="grid">
            <?php foreach ($projects as $p):
                $hasIndex    = file_exists($PROJECTS_ROOT . '/' . $p . '/index.php')
                            || file_exists($PROJECTS_ROOT . '/' . $p . '/public/index.php');
                $hasComposer = file_exists($PROJECTS_ROOT . '/' . $p . '/composer.json');
                $hasGit      = is_dir($PROJECTS_ROOT . '/' . $p . '/.git');
                $matchedVhost = null;
                foreach ($vhosts as $vh) {
                    if (str_contains($vh['server_name'], $p) || str_contains($vh['document_root'], $p)) {
                        $matchedVhost = $vh['server_name'];
                        break;
                    }
                }
            ?>
            <div class="card">
                <div class="card-name"><?= htmlspecialchars($p, ENT_QUOTES, 'UTF-8') ?></div>
                <div class="chips">
                    <?php if ($hasGit): ?>
                        <span class="chip chip-green">git</span>
                    <?php endif; ?>
                    <?php if ($hasComposer): ?>
                        <span class="chip chip-purple">composer</span>
                    <?php endif; ?>
                    <?php if ($hasIndex): ?>
                        <span class="chip chip-blue">index.php</span>
                    <?php endif; ?>
                    <?php if (!$hasGit && !$hasComposer && !$hasIndex): ?>
                        <span class="chip chip-muted">no metadata</span>
                    <?php endif; ?>
                </div>
                <div class="card-divider"></div>
                <?php if ($matchedVhost): ?>
                    <a href="http://<?= htmlspecialchars($matchedVhost, ENT_QUOTES, 'UTF-8') ?>" class="open-link">
                        <?= htmlspecialchars($matchedVhost, ENT_QUOTES, 'UTF-8') ?>
                        <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M2 6h8M6 2l4 4-4 4"/></svg>
                    </a>
                <?php else: ?>
                    <span class="no-vhost">no vhost configured</span>
                <?php endif; ?>
            </div>
            <?php endforeach; ?>
        </div>
        <?php endif; ?>
    </section>

    <!-- Virtual Hosts -->
    <section id="vhosts">
        <div class="sec-head">
            <div class="sec-title">Virtual Hosts</div>
            <?php if (!empty($vhosts)): ?>
            <div class="sec-count"><?= count($vhosts) ?> configured</div>
            <?php endif; ?>
        </div>
        <div class="sec-sub">
            <code><?= htmlspecialchars($DEVPANEL_CONF, ENT_QUOTES, 'UTF-8') ?></code>
            <?php if (!$confEnabled): ?>
                <span class="inline-pill badge-warn">not enabled</span>
            <?php else: ?>
                <span class="inline-pill badge-ok">
                    <span class="dot dot-ok" style="width:5px;height:5px;"></span>
                    enabled
                </span>
            <?php endif; ?>
        </div>

        <?php if (empty($vhosts)): ?>
            <p class="empty">No virtual hosts in devpanel.conf yet.</p>
        <?php else: ?>
        <div class="grid">
            <?php foreach ($vhosts as $vh):
                $rootExists = $vh['document_root'] && is_dir($vh['document_root']);
                $inHosts    = str_contains($hostsContent, $vh['server_name']);
            ?>
            <div class="card">
                <div class="card-name"><?= htmlspecialchars($vh['server_name'], ENT_QUOTES, 'UTF-8') ?></div>
                <?php if ($vh['document_root']): ?>
                <div class="card-path"><?= htmlspecialchars($vh['document_root'], ENT_QUOTES, 'UTF-8') ?></div>
                <?php endif; ?>
                <?php if (!empty($vh['aliases'])): ?>
                <div class="chips">
                    <?php foreach ($vh['aliases'] as $alias): ?>
                    <span class="chip chip-muted"><?= htmlspecialchars($alias, ENT_QUOTES, 'UTF-8') ?></span>
                    <?php endforeach; ?>
                </div>
                <?php endif; ?>
                <div class="chips" style="margin-bottom:0;">
                    <?php if (!$rootExists): ?>
                        <span class="chip" style="background:var(--red-dim);color:var(--red);border-color:rgba(255,69,58,.2);">root missing</span>
                    <?php endif; ?>
                    <?php if (!$inHosts): ?>
                        <span class="chip" style="background:var(--yellow-dim);color:var(--yellow);border-color:rgba(255,214,10,.2);">not in /etc/hosts</span>
                    <?php endif; ?>
                </div>
                <div class="card-divider"></div>
                <a href="http://<?= htmlspecialchars($vh['server_name'], ENT_QUOTES, 'UTF-8') ?>" class="open-link">
                    Open site
                    <svg viewBox="0 0 12 12" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M2 6h8M6 2l4 4-4 4"/></svg>
                </a>
            </div>
            <?php endforeach; ?>
        </div>
        <?php endif; ?>
    </section>

    <!-- Issues -->
    <?php if (!empty($issues)): ?>
    <section id="issues">
        <div class="sec-head">
            <div class="sec-title">Issues</div>
            <div class="sec-count"><?= count($issues) ?> item<?= count($issues) !== 1 ? 's' : '' ?></div>
        </div>
        <div class="sec-sub">Items that need attention before everything works correctly.</div>
        <div class="issue-list">
            <?php foreach ($issues as $issue):
                $cls  = match($issue['level']) { 'error' => 'issue-error', 'warning' => 'issue-warning', default => 'issue-info' };
                $icon = match($issue['level']) {
                    'error'   => '<svg class="issue-icon" viewBox="0 0 16 16" fill="currentColor" style="color:var(--red)"><path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm.75 4.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3zm-.75 6.5a.75.75 0 1 0 0-1.5.75.75 0 0 0 0 1.5z"/></svg>',
                    'warning' => '<svg class="issue-icon" viewBox="0 0 16 16" fill="currentColor" style="color:var(--yellow)"><path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm.75 4.25a.75.75 0 0 0-1.5 0v3a.75.75 0 0 0 1.5 0v-3zm-.75 6.5a.75.75 0 1 0 0-1.5.75.75 0 0 0 0 1.5z"/></svg>',
                    default   => '<svg class="issue-icon" viewBox="0 0 16 16" fill="currentColor" style="color:var(--blue)"><path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm-.75 4.25a.75.75 0 0 1 1.5 0v3a.75.75 0 0 1-1.5 0v-3zM8 11.5a.75.75 0 1 1 0-1.5.75.75 0 0 1 0 1.5z"/></svg>',
                };
            ?>
            <div class="issue <?= $cls ?>">
                <?= $icon ?>
                <span class="issue-msg"><?= htmlspecialchars($issue['msg'], ENT_QUOTES, 'UTF-8') ?></span>
            </div>
            <?php endforeach; ?>
        </div>
    </section>
    <?php endif; ?>

    <!-- System -->
    <section id="system">
        <div class="sec-head">
            <div class="sec-title">System</div>
        </div>
        <div class="sec-sub">PHP runtime, server stats, and loaded extensions.</div>

        <div class="grid" style="margin-bottom:14px;">
            <!-- PHP -->
            <div class="card">
                <div class="card-name" style="margin-bottom:14px;">PHP Runtime</div>
                <div class="stat-group">
                    <div class="stat-row"><span class="stat-key">Version</span><span class="stat-val"><?= htmlspecialchars($phpVersion, ENT_QUOTES, 'UTF-8') ?></span></div>
                    <div class="stat-row"><span class="stat-key">SAPI</span><span class="stat-val"><?= htmlspecialchars(php_sapi_name(), ENT_QUOTES, 'UTF-8') ?></span></div>
                    <div class="stat-row"><span class="stat-key">Memory limit</span><span class="stat-val"><?= ini_get('memory_limit') ?></span></div>
                    <div class="stat-row"><span class="stat-key">Upload max</span><span class="stat-val"><?= ini_get('upload_max_filesize') ?></span></div>
                    <div class="stat-row"><span class="stat-key">Max execution</span><span class="stat-val"><?= ini_get('max_execution_time') ?>s</span></div>
                    <div class="stat-row"><span class="stat-key">Display errors</span><span class="stat-val <?= ini_get('display_errors') ? 'stat-ok' : '' ?>"><?= ini_get('display_errors') ? 'On' : 'Off' ?></span></div>
                </div>
            </div>

            <!-- Server -->
            <div class="card">
                <div class="card-name" style="margin-bottom:14px;">Server</div>
                <div class="stat-group">
                    <div class="stat-row"><span class="stat-key">Hostname</span><span class="stat-val"><?= htmlspecialchars($hostname ?: '-', ENT_QUOTES, 'UTF-8') ?></span></div>
                    <?php if ($uptime): ?>
                    <div class="stat-row"><span class="stat-key">Uptime</span><span class="stat-val"><?= htmlspecialchars($uptime, ENT_QUOTES, 'UTF-8') ?></span></div>
                    <?php endif; ?>
                    <?php if ($memTotalMb): ?>
                    <div class="stat-row"><span class="stat-key">RAM total</span><span class="stat-val"><?= $memTotalMb ?> MB</span></div>
                    <div class="stat-row"><span class="stat-key">RAM available</span><span class="stat-val"><?= intdiv($memFree, 1024) ?> MB</span></div>
                    <?php endif; ?>
                    <div class="stat-row"><span class="stat-key">Apache :80</span><span class="stat-val <?= $apacheOk ? 'stat-ok' : 'stat-err' ?>"><?= $apacheOk ? 'running' : 'down' ?></span></div>
                    <div class="stat-row"><span class="stat-key">MySQL :3306</span><span class="stat-val <?= $mysqlOk ? 'stat-ok' : 'stat-err' ?>"><?= $mysqlOk ? 'running' : 'down' ?></span></div>
                </div>
            </div>
        </div>

        <!-- Extensions -->
        <div class="card">
            <div class="card-name">Loaded Extensions <span style="font-weight:400;font-size:12px;color:var(--text-3);font-family:var(--mono);"><?= count($phpModules) ?> modules</span></div>
            <div class="modules">
                <?php foreach ($phpModules as $mod): ?>
                <span class="mod"><?= htmlspecialchars($mod, ENT_QUOTES, 'UTF-8') ?></span>
                <?php endforeach; ?>
            </div>
        </div>
    </section>

</main>

<!-- ── Footer ── -->
<footer class="footer">
    <span>DevPanel</span>
    <span class="footer-sep">/</span>
    <span>php <?= htmlspecialchars($phpVersion, ENT_QUOTES, 'UTF-8') ?></span>
    <span class="footer-sep">/</span>
    <span><?= htmlspecialchars($_SERVER['SERVER_SOFTWARE'] ?? 'Apache', ENT_QUOTES, 'UTF-8') ?></span>
    <span class="footer-sep">/</span>
    <span><?= htmlspecialchars($hostname ?: 'localhost', ENT_QUOTES, 'UTF-8') ?></span>
</footer>

</div><!-- .shell -->
</body>
</html>
