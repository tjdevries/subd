import * as THREE from 'three';
import * as d3 from 'd3';
import * as mermaid from 'mermaid';

document.addEventListener('DOMContentLoaded', () => {
    console.log('Page loaded - Initializing animations and charts');

    // Initialize Three.js Scene
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);

    document.body.appendChild(renderer.domElement);
    console.log('Three.js scene initialized');

    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);
    console.log('Cube added to scene');

    camera.position.z = 5;

    function animate() {
        requestAnimationFrame(animate);
        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;
        renderer.render(scene, camera);
    }
    animate();
    console.log('Animation started');

    // Initialize D3.js Chart (example)
    const data = [30, 86, 168, 281, 303, 365];
    const chart = d3.select('.header-container')
        .append('svg')
        .attr('width', 500)
        .attr('height', 100)
        .append('g')
        .attr('transform', 'translate(0,0)');
    console.log('D3.js chart initialized');

    chart.selectAll('rect')
        .data(data)
        .enter()
        .append('rect')
        .attr('width', 40)
        .attr('height', d => d)
        .attr('x', (d, i) => i * 45)
        .attr('y', d => 100 - d);
    console.log('D3.js chart data bound');

    // Initialize Mermaid.js
    mermaid.initialize({ startOnLoad: true });

    const mermaidChart = `
        graph LR
        A[User visits site] --> B{User logged in?}
        B -- Yes --> C[Show home page]
        B -- No --> D[Prompt Login]
    `;

    const mermaidDiv = document.createElement('div');
    mermaidDiv.classList.add('mermaid');
    mermaidDiv.textContent = mermaidChart;
    document.body.appendChild(mermaidDiv);

    console.log('Mermaid.js chart initialized');
});