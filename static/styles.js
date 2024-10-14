import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

console.log('Initializing visualizations...');

// Create a Three.js scene with stars and animations
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

const geometry = new THREE.SphereGeometry(0.1, 32, 32);
const material = new THREE.MeshBasicMaterial({color: 0xffffff});
const stars = [];

console.log('Creating stars...');
for (let i = 0; i < 1000; i++) {
  const star = new THREE.Mesh(geometry, material);
  star.position.x = THREE.MathUtils.randFloatSpread(200);
  star.position.y = THREE.MathUtils.randFloatSpread(200);
  star.position.z = THREE.MathUtils.randFloatSpread(200);
  scene.add(star);
  stars.push(star);
}
camera.position.z = 5;

function animateStars() {
  requestAnimationFrame(animateStars);

  stars.forEach((star) => {
    star.rotation.x += 0.001;
    star.rotation.y += 0.001;
  });

  renderer.render(scene, camera);
}
animateStars();

console.log('Rendering stars animation...');

// D3.js Example - A spiraling path representing a journey
d3.select("body").append("svg")
  .attr("width", window.innerWidth)
  .attr("height", window.innerHeight)
  .append("path")
  .datum(d3.range(0, 10 * Math.PI, Math.PI / 50))
  .attr("fill", "none")
  .attr("stroke", "steelblue")
  .attr("stroke-width", 2)
  .attr("d", d3.line()
    .x((d) => window.innerWidth / 2 + d * Math.sin(d))
    .y((d) => window.innerHeight / 2 + d * Math.cos(d))
  );

console.log('D3.js journey path created as a spiral..');

// Mermaid.js Example - Journey graph
document.addEventListener('DOMContentLoaded', () => {
  const graphDefinition = `
  graph LR;
    A(Start) --> B{Decision};
    B -->|Yes| C[Advance];
    B -->|No| D[Fallback];
    C --> E{Progress};
    D --> E;
    E --> F{End};
  `;

  console.log('Initializing Mermaid diagram...');
  const graphContainer = document.createElement('div');
  graphContainer.classList.add('mermaid');
  graphContainer.innerHTML = graphDefinition;
  document.body.appendChild(graphContainer);

  mermaid.initialize({ startOnLoad: true });
  console.log('Mermaid graph rendered.');
});

console.log('Visualization complete.');