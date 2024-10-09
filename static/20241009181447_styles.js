// Import necessary libraries

import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

// Initialize mermaid
mermaid.initialize({ startOnLoad: true });

console.log('Mermaid initialized');

// Three.js setup
function initThreeJS() {
    // Set up the scene
    const scene = new THREE.Scene();
    // Camera
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    camera.position.z = 5;
    // Renderer
    const renderer = new THREE.WebGLRenderer({ alpha: true });
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    console.log('Three.js scene initialized');

    // Animation: Rotating Cube
    const geometry = new THREE.BoxGeometry(1, 1, 1);
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
    const cube = new THREE.Mesh(geometry, material);
    scene.add(cube);

    console.log('Cube added to Three.js scene');

    function animate() {
        requestAnimationFrame(animate);
        cube.rotation.x += 0.01;
        cube.rotation.y += 0.01;
        renderer.render(scene, camera);
        console.log('Animating cube');
    }
    animate();
}

// D3.js setup
function initD3JS() {
    // Create a simple bar chart
    const data = [4, 8, 15, 16, 23, 42];
    const width = 420,
        barHeight = 20;

    const x = D3.scaleLinear()
        .domain([0, D3.max(data)])
        .range([0, width]);

    const chart = D3.select('.chart')
        .attr('width', width)
        .attr('height', barHeight * data.length);

    const bar = chart.selectAll('g')
        .data(data)
        .enter().append('g')
        .attr('transform', (d, i) => `translate(0,${i * barHeight})`);

    bar.append('rect')
        .attr('width', x)
        .attr('height', barHeight - 1)
        .style('fill', 'steelblue');

    bar.append('text')
        .attr('x', d => x(d) - 3)
        .attr('y', barHeight / 2)
        .attr('dy', '.35em')
        .text(d => d);

    console.log('D3.js chart initialized');
}

// Use mermaid to render a chart
function initMermaid() {
    const graphDefinition = `graph TD;
        A[Compile Me Up] --> B[Wake Me Up Before You Deploy];
        B --> C[System's Ready];
        C --> D[Deploy Now];
        D --> E{System Grows};`;
    document.querySelector('.mermaid').innerHTML = mermaid.mermaidAPI.render('graphDiv', graphDefinition);
    console.log('Mermaid chart created');
}

initThreeJS();
initD3JS();
initMermaid();
