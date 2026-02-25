<?php
declare(strict_types=1);

// ======================================================
// UBUNTU LAMP DASHBOARD
// ======================================================

$projectsPath = "/var/www";
$vhostsPath   = "/etc/apache2/sites-enabled";

$projects = [];
$vhosts = [];
$systemErrors = [];
$vhostIssues = [];
$needsRestart = false;

// ======================================================
// PROJECT SCAN
// ======================================================

if (is_dir($projectsPath) && is_readable($projectsPath)) {
    foreach (scandir($projectsPath) as $dir) {
        if ($dir !== "." && $dir !== ".." && $dir !== "html") {
            $fullPath = $projectsPath . "/" . $dir;
            if (is_dir($fullPath)) {
                $projects[] = $dir;
            }
        }
    }
}

// ======================================================
// SERVICE CHECKS
// ======================================================

function checkPort(string $host, int $port): bool {
    $errno = 0;
    $errstr = '';

    $connection = @fsockopen($host, $port, $errno, $errstr, 1);

    if ($connection !== false) {
        fclose($connection);
        return true;
    }

    return false;
}

// Apache
if (!checkPort("127.0.0.1", 80)) {
    $systemErrors[] = ["level"=>"CRITICAL","msg"=>"Apache is not responding on port 80."];
}

// MySQL
if (!checkPort("127.0.0.1", 3306)) {
    $systemErrors[] = ["level"=>"CRITICAL","msg"=>"MySQL is not responding on port 3306."];
}

// ======================================================
// VHOST VALIDATION
// ======================================================

$serverNames = [];
$serverAliases = [];
$docRoots = [];

if (is_dir($vhostsPath) && is_readable($vhostsPath)) {

    foreach (glob($vhostsPath . "/*.conf") ?: [] as $file) {

        $content = @file_get_contents($file);
        if ($content === false) {
            continue;
        }

        $filename = basename($file);

        // Check VirtualHost block exists
        if (!preg_match("/<VirtualHost\b[^>]*>/i", $content)) {
            $vhostIssues[] = ["level"=>"CRITICAL","msg"=>"Missing <VirtualHost> block","file"=>$filename];
        }

        // Check tag balance
        if (substr_count($content,"<VirtualHost") !== substr_count($content,"</VirtualHost>")) {
            $vhostIssues[] = ["level"=>"CRITICAL","msg"=>"Mismatched <VirtualHost> tags","file"=>$filename];
        }

        // ServerName
        preg_match_all("/^\s*ServerName\s+([^\s#]+)/mi", $content, $matches);
        foreach ($matches[1] ?? [] as $name) {

            $name = trim($name);

            if (!preg_match("/^([a-z0-9\-]+\.)+[a-z]{2,}$/i", $name) && $name !== "localhost") {
                $vhostIssues[] = ["level"=>"WARNING","msg"=>"Invalid ServerName format: $name","file"=>$filename];
            }

            if (isset($serverNames[$name])) {
                $vhostIssues[] = ["level"=>"CRITICAL","msg"=>"Duplicate ServerName: $name","file"=>$filename];
            } else {
                $serverNames[$name] = $filename;
                $vhosts[] = $name;
            }
        }

        // ServerAlias
        preg_match_all("/^\s*ServerAlias\s+(.+)$/mi", $content, $aliasMatches);
        foreach ($aliasMatches[1] ?? [] as $aliasLine) {

            foreach (preg_split("/\s+/", trim($aliasLine)) as $alias) {

                if ($alias === '') {
                    continue;
                }

                if (!preg_match("/^([a-z0-9\-]+\.)+[a-z]{2,}$/i", $alias)) {
                    $vhostIssues[] = ["level"=>"WARNING","msg"=>"Invalid ServerAlias: $alias","file"=>$filename];
                }

                if (isset($serverAliases[$alias])) {
                    $vhostIssues[] = ["level"=>"CRITICAL","msg"=>"Duplicate ServerAlias: $alias","file"=>$filename];
                } else {
                    $serverAliases[$alias] = $filename;
                }
            }
        }

        // DocumentRoot
        preg_match("/^\s*DocumentRoot\s+([^\s#]+)/mi", $content, $docMatch);
        if (!empty($docMatch[1])) {

            $root = trim($docMatch[1], '"');

            if (isset($docRoots[$root])) {
                $vhostIssues[] = ["level"=>"WARNING","msg"=>"Duplicate DocumentRoot: $root","file"=>$filename];
            } else {
                $docRoots[$root] = $filename;
            }

            if (!is_dir($root)) {
                $vhostIssues[] = ["level"=>"WARNING","msg"=>"DocumentRoot does not exist: $root","file"=>$filename];
            }
        }
    }

} else {
    $vhostIssues[] = ["level"=>"CRITICAL","msg"=>"VHosts directory not readable","file"=>$vhostsPath];
}

// ======================================================
// APACHE CONFIGTEST
// ======================================================

$output = [];
$returnCode = 0;

exec("apachectl configtest 2>&1", $output, $returnCode);

if ($returnCode !== 0) {
    foreach ($output as $line) {
        $vhostIssues[] = ["level"=>"CRITICAL","msg"=>$line,"file"=>"apachectl"];
    }
} elseif (!empty($vhostIssues)) {
    $needsRestart = true;
}

$allIssues = array_merge($systemErrors, $vhostIssues);

// ======================================================
// SYSTEM INFO
// ======================================================

$phpVersion = phpversion();
$serverSoftware = $_SERVER['SERVER_SOFTWARE'] ?? "Unknown";
$hostname = gethostname();
?>
<!doctype html>
<html>
<head>
<meta charset="utf-8">
<title>LAMP</title>
</head>
<body>

<h1>LAMP</h1>
<p>
<?php echo htmlspecialchars((string)$hostname, ENT_QUOTES, 'UTF-8'); ?>
•
PHP <?php echo htmlspecialchars($phpVersion, ENT_QUOTES, 'UTF-8'); ?>
</p>

<h2>Projects</h2>
<?php foreach($projects as $p): ?>
<a href="http://localhost/<?php echo urlencode($p); ?>">
<?php echo htmlspecialchars($p, ENT_QUOTES, 'UTF-8'); ?>
</a><br>
<?php endforeach; ?>

<h2>Virtual Hosts</h2>
<?php if(!empty($vhosts)): ?>
<?php foreach($vhosts as $v): ?>
<a href="http://<?php echo htmlspecialchars($v, ENT_QUOTES, 'UTF-8'); ?>">
<?php echo htmlspecialchars($v, ENT_QUOTES, 'UTF-8'); ?>
</a><br>
<?php endforeach; ?>
<?php else: ?>
<span>No VirtualHosts detected</span>
<?php endif; ?>

<h2>System Status</h2>

<?php if(empty($allIssues)): ?>
<div>All services and VirtualHosts are valid.</div>
<?php else: ?>
<?php foreach($allIssues as $issue): ?>
<div>
<strong><?php echo htmlspecialchars($issue['level'], ENT_QUOTES, 'UTF-8'); ?>:</strong>
<?php echo htmlspecialchars($issue['msg'], ENT_QUOTES, 'UTF-8'); ?>
<?php if(!empty($issue['file'])): ?>
<em>(<?php echo htmlspecialchars($issue['file'], ENT_QUOTES, 'UTF-8'); ?>)</em>
<?php endif; ?>
</div>
<?php endforeach; ?>
<?php endif; ?>

<?php if($needsRestart): ?>
<div>
Configuration valid but restart may be required:
<pre>sudo systemctl reload apache2</pre>
</div>
<?php endif; ?>

<div>
<?php echo htmlspecialchars($serverSoftware, ENT_QUOTES, 'UTF-8'); ?>
</div>

</body>
</html>
