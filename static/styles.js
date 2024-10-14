// Import necessary libraries
import * as THREE from 'three';
import * as d3 from 'd3';
import * as mermaid from 'mermaid';

document.addEventListener('DOMContentLoaded', () => {
    console.log('DOM fully loaded and parsed');

    // Initialize Mermaid for chart rendering
    mermaid.initialize({ startOnLoad: true });
    console.log('Mermaid initialized');

    // Create a Three.js scene
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    // Add a spinning cube to the scene
    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);

    camera.position.z = 5;

    const animate = function() {
        requestAnimationFrame(animate);

        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;

        renderer.render(scene, camera);
    };

    animate();
    console.log('Three.js scene with spinning cube initialized');

    // Using D3.js to create interactive visualizations
    const data = [12, 31, 22, 17, 25, 18, 29, 14, 9];
    const width = 500,
        height = 500;

    const svg = d3.select('body').append('svg')
        .attr('width', width)
        .attr('height', height);

    svg.selectAll('circle')
        .data(data)
        .enter()
        .append('circle')
        .attr('cx', (d, i) => i * 50 + 25)
        .attr('cy', height / 2)
        .attr('r', d => d)
        .style('fill', 'steelblue');

    console.log('D3.js circles chart created');

    // Mermaid flowchart example
    const graphDefinition = `
        graph TD;
        A[Start] --> B[Do something fun with D3];
        A --> C[Create a scene with Three.js];
        C --> D[Spin the 3D Cube];
        B --> D;
    `;

    const graphDiv = document.createElement('div');
    graphDiv.className = 'mermaid';
    graphDiv.textContent = graphDefinition;
    document.body.appendChild(graphDiv);

    console.log('Mermaid flowchart added to the DOM');

    mermaid.contentLoaded();

    console.log('Animation and visual effects setup complete.');
});

// "Only D [Instrumental]" - Translates to visual animations and interactions displaying dynamic and rhythmic vibes.
// This script aims to fill the page with lively animations, interactive charts, and an engaging 3D experience.