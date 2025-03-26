import * as THREE from 'three';
import * as d3 from 'd3';
import * as mermaid from 'mermaid';

console.log('Initializing 3D scene with Three.js');

// Setup Three.js scene
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);

camera.position.z = 5;

function animate() {
    requestAnimationFrame(animate);
    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;
    renderer.render(scene, camera);
    console.log('Animating cube rotation');
}

animate();

console.log('Setup D3.js elements for economic plots');

// Setup a simple D3 chart
const data = [10, 15, 20, 25, 30];
const svg = d3.select('body').append('svg')
    .attr('width', 500)
    .attr('height', 300);

svg.selectAll('rect')
    .data(data)
    .enter()
    .append('rect')
    .attr('x', (d, i) => i * 30)
    .attr('y', d => 300 - d * 10)
    .attr('width', 25)
    .attr('height', d => d * 10)
    .attr('fill', 'blue');

console.log('Bar chart rendered with D3.js');

console.log('Configuring Mermaid.js for economic flow diagrams');

// Mermaid chart example
mermaid.initialize({startOnLoad:true});
const mermaidDiv = document.createElement('div');
mermaidDiv.innerHTML = `
    graph TD;
    A[Start] --> B{Fix Economy};
    B --> C[Increase Jobs];
    B --> D[Reduce Deficit];
    B --> E{Decision}
    E -->|Yes| F[Invest in Tech]
    E -->|No| G[Cut Taxes]
    `;
document.body.appendChild(mermaidDiv);
mermaid.contentLoaded();
console.log('Mermaid chart rendered with economic concepts');

console.log('JavaScript animation and visualization setup complete.');
