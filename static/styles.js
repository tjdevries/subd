// styles.js file
import * as THREE from 'three';
import * as d3 from 'd3';
import mermaid from 'mermaid';

// Initializing mermaid
mermaid.initialize({ startOnLoad: true });

console.log('Mermaid initialized for rendering charts.');

// Base setup for Three.js
function initThreeJS() {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    console.log('Three.js scene and camera initialized.');

    // Create a cube with glowing edges
    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00, wireframe: true });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);
    camera.position.z = 5;

    console.log('Cube added to the scene.');

    // Animation function
    function animate() {
        requestAnimationFrame(animate);
        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;
        renderer.render(scene, camera);
    }

    animate();
    console.log('Animation started for Three.js cube.');
}

initThreeJS();

// Create charts with d3.js
function generateD3Chart() {
    const data = [4, 8, 15, 16, 23, 42];
    const width = 420, barHeight = 20;
    const x = d3.scaleLinear().domain([0, d3.max(data)]).range([0, width]);
    const chart = d3.select('.chart-container').append('svg')
        .attr('width', width)
        .attr('height', barHeight * data.length);

    const bar = chart.selectAll('g')
        .data(data)
        .enter().append('g')
        .attr('transform', (d, i) => `translate(0,${i * barHeight})`);

    bar.append('rect')
        .attr('width', x)
        .attr('height', barHeight - 1);

    bar.append('text')
        .attr('x', d => x(d) - 3)
        .attr('y', barHeight / 2)
        .attr('dy', '.35em')
        .text(d => d);

    console.log('D3.js chart generated.');
}

generateD3Chart();

console.log('Chart generation finished.');

// Implement an animation of unwrapping presents
document.querySelectorAll('.unplayed-song').forEach(song => {
    song.addEventListener('mouseenter', () => {
        song.style.transition = 'transform 0.5s ease-in-out, background-color 0.5s ease-in-out';
        song.style.transform = 'scale(1.05)';
        song.style.backgroundColor = '#f0f8ff';
        console.log('Song hovered: ', song.querySelector('.song-title').innerText);
    });

    song.addEventListener('mouseleave', () => {
        song.style.transform = 'scale(1)';
        song.style.backgroundColor = '#fff';
    });

    console.log('Mouse enter and leave events added to unplayed-song elements.');
});

// Rendering chart with Mermaid.js
const mermaidChart = `
    graph TD;
    A[Christmas Morning] --> B(Open Presents);
    A --> C[Snow Bright];
    B --> D[Discover Knife];
    D --> E[Curious Exploration];
    E --> F[Dreams fulfilled?]
`;

document.addEventListener('DOMContentLoaded', () => {
    const element = document.createElement('div');
    element.className = 'mermaid';
    element.innerHTML = mermaidChart;
    document.body.appendChild(element);
    mermaid.contentLoaded();

    console.log('Mermaid.js chart rendered.');
});