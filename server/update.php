<?php
/**
 * VopecsPrinter Update Endpoint
 *
 * Place this file at: /vopecsprinter/update/index.php
 * Or configure your web server to route /vopecsprinter/update/* to this file
 *
 * URL Format: /vopecsprinter/update/{target}/{arch}/{current_version}
 * Example: /vopecsprinter/update/darwin/aarch64/1.0.0
 */

header('Content-Type: application/json');

// Parse the request path
$path = $_SERVER['REQUEST_URI'];
$parts = explode('/', trim($path, '/'));

// Get parameters (adjust indices based on your URL structure)
$target = $parts[count($parts) - 3] ?? 'darwin';  // darwin, windows, linux
$arch = $parts[count($parts) - 2] ?? 'aarch64';    // aarch64, x86_64
$currentVersion = $parts[count($parts) - 1] ?? '0.0.0';

// Define the latest version info
$latestVersion = '1.0.1';
$releaseNotes = 'Bug fixes and performance improvements';
$pubDate = '2024-12-03T12:00:00Z';

// Base URL for downloads
$baseUrl = 'https://pos.megacaresa.com/vopecsprinter/releases';

// Version comparison
function compareVersions($v1, $v2) {
    $v1Parts = array_map('intval', explode('.', $v1));
    $v2Parts = array_map('intval', explode('.', $v2));

    for ($i = 0; $i < max(count($v1Parts), count($v2Parts)); $i++) {
        $p1 = $v1Parts[$i] ?? 0;
        $p2 = $v2Parts[$i] ?? 0;
        if ($p1 > $p2) return 1;
        if ($p1 < $p2) return -1;
    }
    return 0;
}

// Check if update is needed
if (compareVersions($latestVersion, $currentVersion) <= 0) {
    // No update available - return 204 No Content
    http_response_code(204);
    exit;
}

// Platform-specific URLs and signatures
// IMPORTANT: Replace SIGNATURE with actual signature from build
$platforms = [
    'darwin-aarch64' => [
        'url' => "$baseUrl/VopecsPrinter_{$latestVersion}_aarch64.app.tar.gz",
        'signature' => file_get_contents(__DIR__ . "/signatures/darwin-aarch64.sig") ?: ''
    ],
    'darwin-x86_64' => [
        'url' => "$baseUrl/VopecsPrinter_{$latestVersion}_x64.app.tar.gz",
        'signature' => file_get_contents(__DIR__ . "/signatures/darwin-x86_64.sig") ?: ''
    ],
    'windows-x86_64' => [
        'url' => "$baseUrl/VopecsPrinter_{$latestVersion}_x64-setup.nsis.zip",
        'signature' => file_get_contents(__DIR__ . "/signatures/windows-x86_64.sig") ?: ''
    ],
    'linux-x86_64' => [
        'url' => "$baseUrl/VopecsPrinter_{$latestVersion}_amd64.AppImage.tar.gz",
        'signature' => file_get_contents(__DIR__ . "/signatures/linux-x86_64.sig") ?: ''
    ]
];

$platformKey = "$target-$arch";

if (!isset($platforms[$platformKey])) {
    http_response_code(204);
    exit;
}

$response = [
    'version' => $latestVersion,
    'notes' => $releaseNotes,
    'pub_date' => $pubDate,
    'platforms' => [
        $platformKey => $platforms[$platformKey]
    ]
];

echo json_encode($response, JSON_PRETTY_PRINT);
