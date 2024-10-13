// Import necessary libraries
import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

// Initialize Meridian for charts
mermaid.initialize({ startOnLoad: true });
console.log('Mermaid.js initialized');

// Scene setup for three.js animation
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Create a rotating cube
const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);
camera.position.z = 5;

// Animating the cube
console.log('Adding rotation animation to cube');
function animate() {
    requestAnimationFrame(animate);
    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;
    renderer.render(scene, camera);
}
animate();

// Create random data set for D3 chart
const data = Array.from({ length: 10 }, () => Math.floor(Math.random() * 100));

// D3.js bar chart
console.log('Creating a D3.js bar chart');
const svg = D3.select("body")
    .append("svg")
    .attr("width", 500)
    .attr("height", 300);

svg.selectAll("rect")
    .data(data)
    .enter()
    .append("rect")
    .attr("x", (d, i) => i * 50)
    .attr("y", (d) => 300 - d * 3)
    .attr("width", 40)
    .attr("height", (d) => d * 3)
    .attr("fill", "#eb4034");

// Log data being used for chart
console.log('D3 chart data:', data);

// Mermaid chart (example: flowchart)
console.log('Creating a mermaid flowchart');
const chartDefinition = "graph LR\nA[Start] --> B{Is it good?}\nB -->|Yes| C[Proceed]\nB -->|No| D[Go back]";
mermaid.render('graphDiv', chartDefinition, function(svgCode) {
    document.getElementById('chart').innerHTML = svgCode;
});

// Ensure the directory for Mermaid is created in the HTML
document.body.innerHTML += '<div id="chart"></div>';
console.log('HTML for Mermaid chart created');