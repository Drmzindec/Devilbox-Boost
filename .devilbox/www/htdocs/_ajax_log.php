<?php
require '../config.php';

if (!loadClass('Helper')->isLoggedIn()) {
	header('HTTP/1.1 403 Forbidden');
	exit;
}

$logFile = '/var/log/php/php-errors.log';
$lines = isset($_GET['lines']) ? min(max((int)$_GET['lines'], 10), 5000) : 100;
$offset = isset($_GET['offset']) ? max((int)$_GET['offset'], 0) : 0;

if ($_SERVER['REQUEST_METHOD'] === 'POST' && isset($_GET['action']) && $_GET['action'] === 'clear') {
	if (is_writable($logFile)) {
		file_put_contents($logFile, '');
		echo json_encode(array('status' => 'ok', 'message' => 'Log cleared'));
	} else {
		echo json_encode(array('status' => 'error', 'message' => 'Log file not writable'));
	}
	exit;
}

if (!file_exists($logFile) || !is_readable($logFile)) {
	echo json_encode(array(
		'status' => 'empty',
		'lines' => array(),
		'size' => 0,
		'offset' => 0,
		'message' => 'No error log file found. Errors will appear here once PHP logs them to /var/log/php/php-errors.log'
	));
	exit;
}

$size = filesize($logFile);

if ($offset > 0 && $offset >= $size) {
	echo json_encode(array(
		'status' => 'ok',
		'lines' => array(),
		'size' => $size,
		'offset' => $size
	));
	exit;
}

$content = '';
if ($offset > 0) {
	$fp = fopen($logFile, 'r');
	fseek($fp, $offset);
	$content = fread($fp, $size - $offset);
	fclose($fp);
} else {
	$allLines = file($logFile, FILE_IGNORE_NEW_LINES);
	$total = count($allLines);
	$start = max(0, $total - $lines);
	$allLines = array_slice($allLines, $start);
	$content = implode("\n", $allLines);
	$offset = $size - strlen($content);
	if ($offset < 0) $offset = 0;
}

$outputLines = array_filter(explode("\n", $content), function($l) { return $l !== ''; });

echo json_encode(array(
	'status' => 'ok',
	'lines' => array_values($outputLines),
	'size' => $size,
	'offset' => $size
));
