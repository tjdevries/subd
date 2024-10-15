import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

console.log('Initializing fabulous animations and interactivity');

// Initialize 3D Scene
console.log('Setting up Three.js Scene');
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Add Cube
console.log('Adding animated cube to the scene');
const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);
camera.position.z = 5;

// Animation loop
function animate() {
    console.log('Animating cube');
    requestAnimationFrame(animate);
    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;
    renderer.render(scene, camera);
}
animate();

// Initialize Mermaid
console.log('Initializing Mermaid for charts');
mermaid.initialize({ startOnLoad: true });
const graphDefinition = `graph TD;
    A[Neo Vim in Your Face] -->|Flows| B[Fast-paced Code]
    B --> C{Syntax Highlighting}
    C -->|Regex Mastery| D[Split Screen Coding]
    C --> E[Plugin Customization]
    E --> F[Command Line Steering]
    B -->G[Vim Divine Navigation]`;

mermaid.render('chart', graphDefinition, (svgCode) => {
    document.body.innerHTML += svgCode;
    console.log('Mermaid chart rendered');
});

// Tooltip interaction with D3
console.log('Creating tooltips with D3.js');
D3.selectAll('.song-title').on('mouseover', function(event) {
    const songTitle = D3.select(this).text();
    D3.select('body').append('div')
      .attr('class', 'tooltip')
      .style('left', (event.pageX + 5) + 'px')
      .style('top', (event.pageY - 28) + 'px')
      .text(`Listen to ${songTitle}`);
    console.log(`Hovered over ${songTitle}`);
}).on('mouseout', function() {
    D3.select('.tooltip').remove();
});

console.log('All animations and interactions set up!');