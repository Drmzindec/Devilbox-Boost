<?php require '../config.php'; ?>
<?php loadClass('Helper')->authPage(); ?>
<!DOCTYPE html>
<html lang="en">
	<head>
		<?php echo loadClass('Html')->getHead(); ?>
		<style>
			#log-container {
				background-color: #1a1a1a;
				color: #c8c8c8;
				font-family: 'SFMono-Regular', 'Consolas', 'Liberation Mono', 'Menlo', monospace;
				font-size: 12px;
				line-height: 1.5;
				padding: 15px;
				border-radius: 4px;
				height: 70vh;
				overflow-y: auto;
				white-space: pre-wrap;
				word-wrap: break-word;
				border: 1px solid #333;
			}
			#log-container .log-line {
				padding: 1px 0;
				border-bottom: 1px solid rgba(255,255,255,0.03);
			}
			#log-container .log-error {
				color: #ff6b6b;
			}
			#log-container .log-warning {
				color: #ffc107;
			}
			#log-container .log-notice {
				color: #64b5f6;
			}
			#log-container .log-fatal {
				color: #ff4444;
				font-weight: bold;
			}
			#log-container .log-stack {
				color: #888;
				padding-left: 20px;
			}
			#log-container .log-date {
				color: #888;
			}
			.log-controls {
				margin-bottom: 15px;
				display: flex;
				align-items: center;
				gap: 10px;
				flex-wrap: wrap;
			}
			.log-controls .btn {
				font-size: 13px;
			}
			.log-status {
				font-size: 12px;
				color: #888;
				margin-left: auto;
			}
			.log-status .live-dot {
				display: inline-block;
				width: 8px;
				height: 8px;
				border-radius: 50%;
				background-color: #4caf50;
				margin-right: 5px;
				animation: pulse 2s infinite;
			}
			.log-status .paused-dot {
				background-color: #ffc107;
				animation: none;
			}
			@keyframes pulse {
				0%, 100% { opacity: 1; }
				50% { opacity: 0.3; }
			}
			#log-empty {
				color: #666;
				text-align: center;
				padding: 60px 20px;
				font-size: 14px;
			}
			.log-line-count {
				font-size: 12px;
				color: #888;
			}
		</style>
	</head>

	<body>
		<?php echo loadClass('Html')->getNavbar(); ?>

		<div class="container-fluid">

			<div class="row">
				<div class="col-md-12">
					<h4>PHP Error Log</h4>
					<p><small>Live tail of <code>/var/log/php/php-errors.log</code> — errors from all your projects appear here.</small></p>

					<div class="log-controls">
						<div class="btn-group btn-group-sm">
							<button id="btn-pause" class="btn btn-outline-warning" onclick="togglePause()">
								<i class="fa fa-pause"></i> Pause
							</button>
							<button class="btn btn-outline-info" onclick="scrollToBottom()">
								<i class="fa fa-arrow-down"></i> Bottom
							</button>
							<button class="btn btn-outline-danger" onclick="clearLog()">
								<i class="fa fa-trash"></i> Clear
							</button>
						</div>

						<select id="lines-select" class="form-control form-control-sm" style="width: auto; display: inline-block;" onchange="changeLines()">
							<option value="50">Last 50 lines</option>
							<option value="100" selected>Last 100 lines</option>
							<option value="250">Last 250 lines</option>
							<option value="500">Last 500 lines</option>
							<option value="1000">Last 1000 lines</option>
						</select>

						<select id="refresh-select" class="form-control form-control-sm" style="width: auto; display: inline-block;" onchange="changeRefresh()">
							<option value="2000" selected>Refresh: 2s</option>
							<option value="5000">Refresh: 5s</option>
							<option value="10000">Refresh: 10s</option>
							<option value="30000">Refresh: 30s</option>
						</select>

						<span class="log-status">
							<span id="status-dot" class="live-dot"></span>
							<span id="status-text">Live</span>
							&nbsp;|&nbsp;
							<span id="line-count" class="log-line-count">0 lines</span>
							&nbsp;|&nbsp;
							<span id="file-size" class="log-line-count">0 KB</span>
						</span>
					</div>

					<div id="log-container">
						<div id="log-empty">Loading error log...</div>
					</div>
				</div>
			</div>
		</div>

		<?php echo loadClass('Html')->getFooter(); ?>

		<script>
		var paused = false;
		var autoScroll = true;
		var refreshInterval = 2000;
		var initialLines = 100;
		var currentOffset = 0;
		var timer = null;
		var logContainer = document.getElementById('log-container');

		function formatLine(line) {
			var div = document.createElement('div');
			div.className = 'log-line';

			if (/Stack trace:|#[0-9]+ /.test(line)) {
				div.className += ' log-stack';
			} else if (/Fatal error|PHP Fatal/i.test(line)) {
				div.className += ' log-fatal';
			} else if (/Warning|PHP Warning/i.test(line)) {
				div.className += ' log-warning';
			} else if (/Notice|PHP Notice|Deprecated/i.test(line)) {
				div.className += ' log-notice';
			} else if (/Error|PHP Error|Parse error/i.test(line)) {
				div.className += ' log-error';
			}

			var escaped = line.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
			escaped = escaped.replace(/\[(\d{2}-\w{3}-\d{4} [\d:]+[^\]]*)\]/g, '<span class="log-date">[$1]</span>');
			div.innerHTML = escaped;
			return div;
		}

		function scrollToBottom() {
			logContainer.scrollTop = logContainer.scrollHeight;
			autoScroll = true;
		}

		logContainer.addEventListener('scroll', function() {
			var atBottom = logContainer.scrollTop + logContainer.clientHeight >= logContainer.scrollHeight - 30;
			autoScroll = atBottom;
		});

		function fetchLog(initial) {
			if (paused) return;

			var url = '_ajax_log.php?lines=' + initialLines;
			if (!initial && currentOffset > 0) {
				url = '_ajax_log.php?offset=' + currentOffset;
			}

			var xhttp = new XMLHttpRequest();
			xhttp.onreadystatechange = function() {
				if (this.readyState == 4 && this.status == 200) {
					try {
						var data = JSON.parse(this.responseText);
						updateLog(data, initial);
					} catch(e) {}
				}
			};
			xhttp.open('GET', url, true);
			xhttp.send();
		}

		function updateLog(data, initial) {
			if (data.status === 'empty') {
				document.getElementById('log-empty').textContent = data.message || 'No errors logged yet.';
				document.getElementById('line-count').textContent = '0 lines';
				document.getElementById('file-size').textContent = '0 KB';
				return;
			}

			if (data.lines.length === 0 && !initial) {
				return;
			}

			currentOffset = data.offset;

			if (initial) {
				logContainer.innerHTML = '';
			}

			var existingEmpty = document.getElementById('log-empty');
			if (existingEmpty) existingEmpty.remove();

			for (var i = 0; i < data.lines.length; i++) {
				logContainer.appendChild(formatLine(data.lines[i]));
			}

			var totalLines = logContainer.querySelectorAll('.log-line').length;
			document.getElementById('line-count').textContent = totalLines + ' lines';
			document.getElementById('file-size').textContent = (data.size / 1024).toFixed(1) + ' KB';

			// Trim excess lines from the top
			while (totalLines > 5000) {
				var first = logContainer.querySelector('.log-line');
				if (first) first.remove();
				totalLines--;
			}

			if (autoScroll) {
				scrollToBottom();
			}
		}

		function togglePause() {
			paused = !paused;
			var btn = document.getElementById('btn-pause');
			var dot = document.getElementById('status-dot');
			var text = document.getElementById('status-text');

			if (paused) {
				btn.innerHTML = '<i class="fa fa-play"></i> Resume';
				btn.className = 'btn btn-outline-success';
				dot.className = 'live-dot paused-dot';
				text.textContent = 'Paused';
			} else {
				btn.innerHTML = '<i class="fa fa-pause"></i> Pause';
				btn.className = 'btn btn-outline-warning';
				dot.className = 'live-dot';
				text.textContent = 'Live';
				fetchLog(false);
			}
		}

		function clearLog() {
			if (!confirm('Clear the PHP error log?')) return;

			var xhttp = new XMLHttpRequest();
			xhttp.onreadystatechange = function() {
				if (this.readyState == 4 && this.status == 200) {
					logContainer.innerHTML = '<div id="log-empty">Log cleared. New errors will appear here.</div>';
					currentOffset = 0;
					document.getElementById('line-count').textContent = '0 lines';
					document.getElementById('file-size').textContent = '0 KB';
				}
			};
			xhttp.open('POST', '_ajax_log.php?action=clear', true);
			xhttp.send();
		}

		function changeLines() {
			initialLines = parseInt(document.getElementById('lines-select').value);
			currentOffset = 0;
			fetchLog(true);
		}

		function changeRefresh() {
			refreshInterval = parseInt(document.getElementById('refresh-select').value);
			clearInterval(timer);
			timer = setInterval(function() { fetchLog(false); }, refreshInterval);
		}

		fetchLog(true);
		timer = setInterval(function() { fetchLog(false); }, refreshInterval);
		</script>
	</body>
</html>
