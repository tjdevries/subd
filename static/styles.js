import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

console.log('JavaScript loaded: Initializing animations and visualizations.');

// Initialize mermaid for charts
mermaid.initialize({ startOnLoad: true });
console.log('Mermaid initialized for creating charts.');

// Set a neon theme for the site
const neonStyle = document.createElement('style');
neonStyle.innerHTML = `
  body {
    background-color: #000;
    color: #0ff;
    font-family: 'Lucida Console', Monaco, monospace;
  }
  a {
    color: #0f0;
    text-shadow: 0 0 3px #0f0, 0 0 5px #fff;
  }
  .header h1, .sub-header {
    text-shadow: 0 0 5px #0ff, 0 0 10px #00f;
  }
  .song-title, .song-id, .song-tags, .song-description {
    text-shadow: 0 0 3px #0ff;
  }
`;
document.head.appendChild(neonStyle);
console.log('Applied neon theme to the site.');

// Create a 3D animation using three.js
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
const light = new THREE.PointLight(0xFFFF00);
const cubeGeometry = new THREE.BoxGeometry();
const cubeMaterial = new THREE.MeshBasicMaterial({ color: 0x0ff });
const cube = new THREE.Mesh(cubeGeometry, cubeMaterial);
document.body.appendChild(renderer.domElement);

light.position.set(10, 10, 10);
scene.add(light);
camera.position.z = 5;
scene.add(cube);

console.log('3D scene setup complete. Adding 3D elements to the scene.');

function animate3D() {
  requestAnimationFrame(animate3D);
  cube.rotation.x += 0.01;
  cube.rotation.y += 0.01;
  renderer.render(scene, camera);
}

renderer.setSize(window.innerWidth, window.innerHeight);
animate3D();
console.log('Animated 3D cube rotation started.');

// Create a bar chart using D3.js
console.log('Generating bar chart with D3.js');

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
  .attr('transform', (d, i) => `translate(0,${i * barHeight})`);

bar.append('rect')
  .attr('width', x)
  .attr('height', barHeight - 1)
  .style('fill', '#0f0');

bar.append('text')
  .attr('x', d => x(d) - 3)
  .attr('y', barHeight / 2)
  .attr('dy', '.35em')
  .text(d => d);

console.log('Bar chart successfully created.');

// Example mermaid chart
const mermaidElem = document.createElement('div');
mermaidElem.classList.add('mermaid');
mermaidElem.innerHTML = `graph TD;
    A[Start] --> B{Is it?};
    B -->|Yes| C[Great];
    B -->|No| D[Not great];
    C --> E[Do it again];`;
document.body.appendChild(mermaidElem);
mermaid.init(undefined, mermaidElem);
console.log('Mermaid chart created and displayed.');