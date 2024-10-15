// Import required modules
import * as THREE from 'three';
import * as d3 from 'd3';
import mermaid from 'mermaid';

console.log('Initializing Three.js scene...');

// THREE.js Initialization
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Create a rotating cube in the scene
const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);

camera.position.z = 5;

const animateThreeJSScene = function () {
    requestAnimationFrame(animateThreeJSScene);
    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;
    renderer.render(scene, camera);
    console.log('Rendering Three.js scene...');
};

animateThreeJSScene();

console.log('Initializing D3.js chart...');

// D3.js Example: Creating a Fun Bar Chart
const data = [30, 86, 168, 281, 303, 365];
const x = d3.scaleLinear()
    .domain([0, d3.max(data)])
    .range([0, 500]);

const chart = d3.select("body")
    .append("svg")
    .attr("width", 500)
    .attr("height", 20 * data.length);

const bar = chart.selectAll("g")
    .data(data)
    .enter().append("g")
    .attr("transform", (d, i) => `translate(0,${i * 20})`);

bar.append("rect")
    .attr("width", x)
    .attr("height", 19);

bar.append("text")
    .attr("x", d => x(d) - 3)
    .attr("y", 9.5)
    .attr("dy", ".35em")
    .text(d => d);

console.log('D3.js chart created with dynamic data.');

console.log('Initializing Mermaid chart...');

// Mermaid.js Initialization
mermaid.initialize({startOnLoad:true});

// Create a simple flowchart using Mermaid
const mermaidCode = `
  graph TD;
    A[Get your wig on] --> B[Chillin' out];
    A --> C[Time to jive];
    B --> D[Cowabunga ride the waves];
    C --> D;
`;

const mermaidElement = document.createElement('div');
mermaidElement.classList.add('mermaid');
mermaidElement.innerHTML = mermaidCode;
document.body.appendChild(mermaidElement);

mermaid.contentLoaded();
console.log('Mermaid chart rendered.');