import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

console.log('Initializing the fun and animated page...');

// Setup the scene for Three.js animations
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Creating a rotating cube
const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshBasicMaterial({color: 0x00ff00});
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);
camera.position.z = 5;

console.log('Cube created and added to scene.');

// Animation function for the cube
function animate() {
    requestAnimationFrame(animate);
    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;
    renderer.render(scene, camera);
    console.log('Cube is rotating...');
}
animate();

// Setup for D3.js chart
const data = [4, 8, 15, 16, 23, 42];
const width = 420;
const barHeight = 20;

const x = D3.scaleLinear()
    .domain([0, D3.max(data)])
    .range([0, width]);

const chart = D3.select('body')
    .append('svg')
    .attr('width', width)
    .attr('height', barHeight * data.length);

const bar = chart.selectAll('g')
    .data(data)
    .enter().append('g')
    .attr('transform', function(d, i) { return 'translate(0,' + i * barHeight + ')'; });

bar.append('rect')
    .attr('width', x)
    .attr('height', barHeight - 1);

bar.append('text')
    .attr('x', function(d) { return x(d) - 3; })
    .attr('y', barHeight / 2)
    .attr('dy', '.35em')
    .text(function(d) { return d; });

console.log('Bar chart created with D3.js.');

// Initialize Mermaid
mermaid.initialize({ startOnLoad: true });

// Mermaid.js diagram
const graphDefinition = 'graph TD; A-->B; A-->C; B-->D; C-->D;';

mermaid.mermaidAPI.render('theGraph', graphDefinition, (svgCode) => {
    document.body.innerHTML += svgCode;
    console.log('Mermaid graph rendered.');
});