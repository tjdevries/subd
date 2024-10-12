import * as THREE from 'three';
import * as d3 from 'd3';
import mermaid from 'mermaid';

// Initialize the scene for three.js
console.log('Initializing Three.js scene...');
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Add a rotating kawaii cube
console.log('Creating a rotating cube...');
const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshBasicMaterial({ color: 0xff69b4 }); //Bright Pink for Kawaii
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);
camera.position.z = 5;

function animate() {
    requestAnimationFrame(animate);
    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;
    renderer.render(scene, camera);
}

console.log('Starting animation loop...');
animate();

// Initialize D3 chart (e.g., for songs created over time)
console.log('Generating D3 chart...');
const svg = d3.select('body').append('svg')
    .attr('width', 400)
    .attr('height', 300);

const data = [30, 80, 45, 60, 20, 90, 50];

svg.selectAll('rect')
    .data(data)
    .enter().append('rect')
    .attr('x', (d, i) => i * 45)
    .attr('y', d => 300 - d)
    .attr('width', 40)
    .attr('height', d => d)
    .attr('fill', '#ffb6c1'); // Light Pink

console.log('D3 chart rendering complete.');

// Initialize Mermaid diagram for social interaction
console.log('Configuring Mermaid for interaction chart...');
mermaid.initialize({ startOnLoad: true });
const graphDefinition = `graph TD;
    A[Kawaii Senpai] --> B[Heart Races];
    B --> C[Blushing Cheeks];
    C --> D[UwU Eyes];
    D --> E[Stars Align];
    E --> F[Kawaii Senpai Heeya];
    F --> G[Dreams Never Ending];`;
document.addEventListener('DOMContentLoaded', () => {
    const graphElement = document.createElement('div');
    graphElement.className = 'mermaid';
    graphElement.innerHTML = graphDefinition;
    document.body.appendChild(graphElement);
    mermaid.init();
});

console.log('Mermaid diagram created.');