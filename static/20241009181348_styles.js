import * as THREE from 'three';
import * as d3 from 'd3';
import * as mermaid from 'mermaid';

// Function to initialize a 3D scene using Three.js
document.addEventListener('DOMContentLoaded', () => {
    console.log('Initializing 3D scene with Three.js');
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    const geometry = new THREE.BoxGeometry();
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);

    camera.position.z = 5;

    const animate = function () {
        requestAnimationFrame(animate);

        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;

        renderer.render(scene, camera);
    };

    animate();
    console.log('3D cube animation started!');
});

// Function to draw charts using D3.js
document.addEventListener('DOMContentLoaded', () => {
    console.log('Drawing charts using D3.js');
    const svg = d3.select('body').append('svg')
        .attr('width', 600)
        .attr('height', 400);

    const data = [30, 80, 45, 60, 20, 90, 50];
    const barWidth = 40;
    const barPadding = 5;

    svg.selectAll('rect')
        .data(data)
        .enter()
        .append('rect')
        .attr('x', (d, i) => i * (barWidth + barPadding))
        .attr('y', d => 400 - d * 4)
        .attr('width', barWidth)
        .attr('height', d => d * 4)
        .attr('fill', 'neon');

    console.log('Bar chart created with D3.js!');
});

// Function to render mermaid diagrams
document.addEventListener('DOMContentLoaded', () => {
    console.log('Initializing Mermaid diagrams');
    mermaid.initialize({ startOnLoad: true });
    const graphDefinition = `graph TD;
        A[AI Songs] --> B{Vote}
        B -->|Positive| C(Vote Count)
        B -->|Negative| D(Song List)`;

    const graphContainer = document.createElement('div');
    graphContainer.className = 'mermaid';
    graphContainer.innerHTML = graphDefinition;
    document.body.appendChild(graphContainer);

    mermaid.contentLoaded();
    console.log('Mermaid diagram rendered!');
});

// Additional animations and logic
document.addEventListener('DOMContentLoaded', () => {
    console.log('Adding neon-inspired animations');
    document.querySelectorAll('.song-title').forEach(element => {
        element.style.transition = 'color 0.5s ease-in-out';

        element.addEventListener('mouseenter', () => {
            element.style.color = 'lime';
            console.log(`Hovered over: ${element.textContent}`);
        });
        element.addEventListener('mouseleave', () => {
            element.style.color = '';
        });
    });
});