import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

console.log('Initializing trendy hipster aquatic design with 3D animations and charts.');

// Initialize Three.js scene
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

console.log('Three.js renderer added to the DOM.');

// Create a rotating cube
const geometry = new THREE.BoxGeometry();
const material = new THREE.MeshBasicMaterial({ color: 0x0077ff });
const cube = new THREE.Mesh(geometry, material);
scene.add(cube);
camera.position.z = 5;

const animate = function () {
    requestAnimationFrame(animate);

    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;
    
    renderer.render(scene, camera);
    console.log('Rotating cube animation frame rendered.');
};

animate();

// Set up Mermaid.js for chart rendering
mermaid.initialize({ startOnLoad: true });
let chartDefinition = `graph TD;
    A[Start] --> B[Hipster Path];
    B --> C{Choices};
    C -->|Trend| D[Trendy Hipster Design];
    C -->|Aquatic| E[Aquatic Fun];
    D --> F[End];
    E --> F[Fun];
`;

console.log('Mermaid chart initialized.');
mermaid.render('mermaid-chart', chartDefinition, (svgCode) => {
    document.getElementById('mermaid').innerHTML = svgCode;
    console.log('Mermaid chart rendered.');
});

// Using D3.js to create a dynamic representation
const svg = D3.select('body').append('svg')
    .attr('width', 600)
    .attr('height', 400);

console.log('D3.js SVG placeholder created.');

let data = [30, 86, 168, 281, 303, 365];

svg.selectAll('rect')
    .data(data)
    .enter()
    .append('rect')
    .attr('x', (d, i) => i * 40)
    .attr('y', (d) => 400 - d)
    .attr('width', 35)
    .attr('height', (d) => d)
    .attr('fill', 'aquamarine')
    .on('mouseover', (d, i) => console.log(`Mouse over bar: ${i}, Value: ${d}`));

console.log('D3.js dynamic bar chart created with interaction.');