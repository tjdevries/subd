import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

console.log('Page Loaded: Sweet and Sour Love Theme with Animations');

// Initialize Three.js scene
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);
camera.position.z = 5;

// Animation loop
function animate() {
    requestAnimationFrame(animate);
    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;
    renderer.render(scene, camera);
    console.log('Animating Cube: x rotation ', cube.rotation.x, ' y rotation ', cube.rotation.y);
}
animate();

// Initialize D3.js chart
const data = [30, 86, 168, 281, 303, 365];
D3.select('.chart')
  .selectAll('div')
  .data(data)
  .enter().append('div')
  .style('width', function(d) { return d + 'px'; })
  .text(function(d) { return d; });
console.log('D3.js Chart Initialized with Data:', data);

// Initialize Mermaid.js diagram
mermaid.initialize({ startOnLoad: true });
const graphDefinition = `graph TD;
    A[Start] --> B{Is it?};
    B -->|Yes| C[Great];
    B -->|No| D[Continue];
    C --> E[End];
    D --> E;
`;
mermaid.parse(graphDefinition);
console.log('Mermaid.js Diagram:', graphDefinition);

document.addEventListener('DOMContentLoaded', function() {
    const app = document.getElementById('mermaid-diagram');
    app.innerHTML = `  
    <div class="mermaid">
        ${graphDefinition}
    </div>`;
    console.log('Mermaid Diagram Rendered');
});

console.log('JavaScript animations and visualizations ready');