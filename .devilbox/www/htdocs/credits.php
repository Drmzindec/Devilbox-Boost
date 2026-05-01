<?php require '../config.php'; ?>
<?php loadClass('Helper')->authPage(); ?>
<!DOCTYPE html>
<html lang="en">
	<head>
		<?php echo loadClass('Html')->getHead(true); ?>
	</head>

	<body>
		<?php echo loadClass('Html')->getNavbar(); ?>

		<div class="container">

			<h1>Credits</h1>
			<br/>
			<br/>

			<div class="row">

				<div class="col-md-6">
					<h2>Devilbox Boost</h2>
					<p>An enhanced fork of <a href="https://github.com/cytopia/devilbox">Devilbox</a> with modern PHP support, updated tools, and quality-of-life improvements.</p>
					<table class="table table-striped ">
						<thead class="thead-inverse ">
							<tr>
								<th>Contributor</th>
								<th>Contributions</th>
								<th>Url</th>
							</tr>
						</thead>
						<tbody>
							<tr>
								<td>Drmzindec</td>
								<td>Boost creator &amp; maintainer</td>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a href="https://github.com/Drmzindec/Devilbox-Boost">Devilbox-Boost</a></td>
							</tr>
							<tr>
								<td>cytopia</td>
								<td>Original Devilbox creator</td>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a href="https://github.com/cytopia">cytopia</a></td>
							</tr>
							<tr>
								<td>Maifz</td>
								<td>Logos</td>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a href="https://github.com/Maifz">Maifz</a></td>
							</tr>
						</tbody>
					</table>
					<p>Want to contribute? See the <a href="https://github.com/Drmzindec/Devilbox-Boost/blob/main/CONTRIBUTING.md">Contributing Guidelines</a>.</p>
				</div>


				<div class="col-md-6">
					<h2>Libraries</h2>
					<p><a href="https://github.com/Drmzindec/Devilbox-Boost">Devilbox Boost</a> includes the following libraries.</p>
					<table class="table table-striped ">
						<thead class="thead-inverse ">
							<tr>
								<th>Vendor</th>
								<th>License</th>
								<th>Url</th>
							</tr>
						</thead>
						<tbody>
							<tr>
								<td>Adminer 5.4.2</td>
								<td>Apache License 2.0 or GPL 2</td>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/vrana/adminer">vrana/adminer</a></td>
							</tr>
							<tr>
								<td>Bootstrap</td>
								<td>MIT</td>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/twbs/bootstrap">twbs/bootstrap</a></td>
							</tr>
							<tr>
								<td>Font Awesome (css)</td>
								<td>MIT</td>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/FortAwesome/Font-Awesome">FortAwesome/Font-Awesome</a></td>
							</tr>
							<tr>
								<td>Font Awesome (fonts)</td>
								<td>SIL OFL 1.1</td>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/FortAwesome/Font-Awesome">FortAwesome/Font-Awesome</a></td>
							</tr>
							<tr>
								<td>OpCache GUI 3.6.0</td>
								<td>MIT</td>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/amnuts/opcache-gui">amnuts/opcache-gui</a></td>
							</tr>
							<tr>
								<td>phpCacheAdmin 2.4.1</td>
								<td>Apache 2.0</td>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/RobiNN1/phpCacheAdmin">RobiNN1/phpCacheAdmin</a></td>
							</tr>
							<tr>
								<td>phpMyAdmin 5.2.3</td>
								<td>GPL 2.0</td>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/phpmyadmin/phpmyadmin">phpmyadmin/phpmyadmin</a></td>
							</tr>
							<tr>
								<td>phpPgAdmin 7.13.0</td>
								<td>GPL 2.0</td>
								<td><i class="fa fa-github-alt" aria-hidden="true"></i> <a target="_blank" href="https://github.com/phppgadmin/phppgadmin">phppgadmin/phppgadmin</a></td>
							</tr>
						</tbody>
					</table>
				</div>

			</div>

		</div><!-- /.container -->

		<?php echo loadClass('Html')->getFooter(); ?>
	</body>
</html>
