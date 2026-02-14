<?php
/**
 * Adminer for Devilbox with prefilled connection (127.0.0.1, root:root)
 */

// Prefill connection details if not already set
if (!isset($_POST['auth'])) {
    $_GET['server'] = $_GET['server'] ?? '127.0.0.1';
    $_GET['username'] = $_GET['username'] ?? 'root';
}

// Include Adminer
include './adminer-5.4.2-en.php';
