// styles.js
import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

console.log('Initializing 3D scene with Three.js');

// Create a Three.js scene
const scene = new THREE.Scene();
console.log('Scene created');

const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
console.log('Camera added to scene');

const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);
console.log('Renderer initialized');

const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);
console.log('Green cube added to the scene');

camera.position.z = 5;

function animate() {
    requestAnimationFrame(animate);
    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;
    console.log('Cube rotated: ', cube.rotation.x, cube.rotation.y);
    renderer.render(scene, camera);
}

animate();

console.log('Setting up data visualization with D3.js');

// Example data for D3.js chart
const data = [
    { year: 2021, pigs: 20 },
    { year: 2022, pigs: 15 },
    { year: 2023, pigs: 5 }
];

const svg = D3.select('body').append('svg')
    .attr('width', 500)
    .attr('height', 300);

const x = D3.scaleBand()
    .domain(data.map(d => d.year))
    .range([0, 500])
    .padding(0.1);

const y = D3.scaleLinear()
    .domain([0, D3.max(data, d => d.pigs)])
    .nice()
    .range([300, 0]);

svg.append('g')
    .selectAll('rect')
    .data(data)
    .enter().append('rect')
    .attr('x', d => x(d.year))
    .attr('y', d => y(d.pigs))
    .attr('width', x.bandwidth())
    .attr('height', d => y(0) - y(d.pigs))
    .attr('fill', 'steelblue');

svg.append('g')
    .attr('transform', 'translate(0,300)')
    .call(D3.axisBottom(x));

svg.append('g')
    .call(D3.axisLeft(y));

console.log('D3.js chart created for pig population over years');

console.log('Configuring Mermaid.js for chart rendering');

// Mermaid configuration and chart
mermaid.initialize({ startOnLoad: true });

const diagram = `graph LR
    A[Not Enough Pigs] --> B(Verse: Empty Pens)
    A --> C(Verse 2: Farmer's Field Alone)
    B --> D[Chorus: Pigs I've Known]
    C --> D
    D --> E(Verse 3: Echoes Fade)
    E --> F{Bridge: Empty Heart}
    F --> G(Verse 4: Lonely Swineherd)
    `;

document.addEventListener('DOMContentLoaded', () => {
    const mermaidDiv = document.createElement('div');
    mermaidDiv.className = 'mermaid';
    mermaidDiv.textContent = diagram;
    document.body.appendChild(mermaidDiv);
    mermaid.init(undefined, mermaidDiv);
    console.log('Mermaid.js diagram initialized');
});