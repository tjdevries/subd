import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

// Initialize Mermaid for diagramming
mermaid.initialize({ startOnLoad: true });
console.log('Mermaid initialized for chart rendering.');

document.addEventListener('DOMContentLoaded', () => {
    console.log('Document fully loaded. Starting animations...');

    // THREE.js: Create a basic scene with some pixelated animations (like a cyberpunk game)
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);
    console.log('Three.js renderer set up and elements appended to DOM.');

    // Basic Cube Animation
    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00, wireframe: true });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);

    camera.position.z = 5;

    function animate() {
        requestAnimationFrame(animate);
        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;
        renderer.render(scene, camera);
        console.log('Animating cube with rotation.');
    }

    animate();

    // D3.js: Add some cyberpunk-style animated charts
    const dataset = [30, 50, 80, 120, 150];
    const svg = D3.select("body").append("svg")
        .attr("width", 500)
        .attr("height", 500);
    console.log('SVG created for D3 charts.');

    svg.selectAll("rect")
        .data(dataset)
        .enter()
        .append("rect")
        .attr("x", (d, i) => i * 101)
        .attr("y", d => 500 - d)
        .attr("width", 100)
        .attr("height", d => d)
        .attr("fill", "teal")
        .attr("stroke", "black")
        .attr("stroke-width", 3);
    console.log('D3 bars created and styled.');

    // Animated chart in Mermaid
    let chartDefinition = `graph TD; 
        A[Start] -->|Cyber Data| B(Decision);
        B -->|Yes| C[Do something cool];
        B -->|No| D[Do something else];`;

    mermaid.render('mermaidChart', chartDefinition, (svgCode) => {
        document.body.innerHTML += svgCode;
        console.log('Mermaid chart rendered and added to DOM.');
    });
});
