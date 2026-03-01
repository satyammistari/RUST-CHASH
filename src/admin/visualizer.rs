use crate::ring::HashRing;

pub fn visualizer_page(ring: &HashRing) -> String {
	let nodes = ring.nodes();
	let mut rows = String::new();
	for (hash, backend) in &nodes {
		rows.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>", hash, backend));
	}

	format!(
		r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<title>Consistent Hash Ring</title>
<style>
body {{ font-family: Arial, sans-serif; margin: 2rem; background: #fafafa; }}
table {{ width: 100%; border-collapse: collapse; }}
th, td {{ border: 1px solid #ccc; padding: 0.5rem; text-align: left; }}
th {{ background: #f0f0f0; }}
</style>
</head>
<body>
<h1>Consistent Hash Ring</h1>
<p>Total virtual nodes: {}</p>
<table>
<thead><tr><th>Hash</th><th>Backend</th></tr></thead>
<tbody>{}</tbody>
</table>
</body>
</html>"#,
		nodes.len(),
		rows
	)
}
