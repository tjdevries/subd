import * as THREE from 'three';
import * as d3 from 'd3';
import * as mermaid from 'mermaid';

// Initialize Three.js scene
console.log('Initializing Three.js scene...');
const scene = new THREE.Scene();

const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

const geometry = new THREE.TorusGeometry(10, 3, 16, 100);
const material = new THREE.MeshBasicMaterial({ color: 0xffff00 });
const torus = new THREE.Mesh(geometry, material);
scene.add(torus);

camera.position.z = 20;

function animate() {
    requestAnimationFrame(animate);
    torus.rotation.x += 0.01;
    torus.rotation.y += 0.01;
    console.log('Animating Torus...');
    renderer.render(scene, camera);
}

animate();

// Initialize D3.js animation
console.log('Initializing D3.js animation...');
const svg = d3.select("body").append("svg").attr("width", 500).attr("height", 300);
const data = [10, 20, 30, 40, 50];

const circles = svg.selectAll("circle")
  .data(data)
  .enter()
  .append("circle")
  .attr("cx", (d, i) => i * 100 + 50)
  .attr("cy", 150)
  .attr("r", d => d)
  .style("fill", "steelblue");

circles.transition()
    .duration(1000)
    .attr("cy", 50)
    .style("fill", "orange")
    .delay((d, i) => i * 500)
    .on("start", function() { console.log('D3.js transition starting...'); });

// Initialize mermaid diagram
console.log('Initializing Mermaid diagram...');
mermaid.initialize({ startOnLoad: true });
document.addEventListener("DOMContentLoaded", function() {
    document.querySelectorAll('.chart').forEach(el => {
        el.innerHTML = `
graph TD;
    A[Theory] --> B(Mystery);
    B --> C[Gravity];
    C -->|Symmetry| D[Space];
    C -->|Symmetry| E[Time];
    F[Equations on Blackboard] --> G[Cosmic Ballet];
    G --> H{Beauty?};
    H --> I[Breakthrough];
    H --> J[Jest];
    `;
        mermaid.contentLoaded();
        console.log('Rendering mermaid diagram...');
    });
});