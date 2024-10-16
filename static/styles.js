import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

// Initializing Mermaid for creating flowcharts
mermaid.initialize({
  startOnLoad: true
});

// Create a Gothic and Evil themed scene with Three.js
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Adding gothic looking cube to the scene
const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshBasicMaterial({ color: 0x000000 });
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);

camera.position.z = 5;

// Animation function for the cube that rotates
function animateCube() {
  requestAnimationFrame(animateCube);
  cube.rotation.x += 0.01;
  cube.rotation.y += 0.01;
  renderer.render(scene, camera);
  console.log('Cube is rotating in a gothic universe');
}

animateCube();

// Adding some D3.js effect for goth-like interactivity
const darkData = [10, 20, 30, 40, 50];

const svg = D3.select("body").append("svg")
              .attr("width", 300)
              .attr("height", 200);

const barHeight = 20;

// Rendering bars with dark colors
const bar = svg.selectAll("g")
  .data(darkData)
  .enter().append("g")
  .attr("transform", (d, i) => `translate(0,${i * barHeight})`);

bar.append("rect")
  .attr("width", d => d * 10)
  .attr("height", barHeight - 1)
  .style("fill", "#330000")
  .style("stroke", "#660000");

bar.append("text")
  .attr("x", d => (d * 10) - 3)
  .attr("y", barHeight / 2)
  .attr("dy", ".35em")
  .text(d => d);

console.log('D3 dark-themed bars added');

// Integrating Mermaid.js for a flowchart
mermaid.contentLoaded();

const chart = `graph TD
    A[Start] --> B{Gothic Path?}
    B -->|Yes| C[Embrace Darkness]
    B -->|No| D[Return]
    C --> E[End]
    D --> F[Retry]
`;

mermaid.render('mermaidChart', chart, (svgCode) => {
  const container = document.createElement('div');
  container.innerHTML = svgCode;
  document.body.appendChild(container);
  console.log('Mermaid chart rendered');
});
