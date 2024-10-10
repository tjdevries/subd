import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

// Initialize mermaid
mermaid.initialize({startOnLoad:true});
console.log('Mermaid initialized for charts rendering.');

// Set up a scene with THREE.js
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshBasicMaterial({color: 0x00ff00, wireframe: true});
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);

camera.position.z = 5;

const animate = function () {
    requestAnimationFrame(animate);

    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;

    renderer.render(scene, camera);
};

console.log('Adding a spinning cube with THREE.js.');
animate();

// Use D3.js to create a fun animation
const data = [10, 20, 30, 40, 50];
const svg = D3.select('body').append('svg')
    .attr('width', 300)
    .attr('height', 300)
    .style('background', 'black');

svg.selectAll('circle')
    .data(data)
    .enter()
    .append('circle')
    .attr('cx', (d, i) => i * 50 + 30)
    .attr('cy', 150)
    .attr('r', 0)
    .style('fill', 'neon')
    .transition()
    .duration(2000)
    .attr('r', (d) => d)
    .text((d) => d);

console.log('D3 circles animated to grow with neon color.');

// Sample mermaid chart
document.body.innerHTML += `
<div class="mermaid">
  graph TD;
      A[Start] --> B{Do Task};
      B -->|Yes| C[Task 1];
      C --> D[End];
      B -->|No| E[Task 2];
      E --> D[End];
</div>
`;

console.log('Mermaid diagram added to the page. Review the flow of the Mermaid graph.');