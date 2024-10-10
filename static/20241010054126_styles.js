import * as THREE from 'three';
import * as D3 from 'd3';
import * as mermaid from 'mermaid';

// Initialize mermaid chart configuration
mermaid.initialize({startOnLoad:true});

window.addEventListener('DOMContentLoaded', () => {
    console.log('DOM fully loaded and parsed');

    // Create a simple D3 bar chart
    const data = [10, 15, 20, 25, 30];
    const width = 500, height = 100;
    const svg = D3.select('body').append('svg')
        .attr('width', width)
        .attr('height', height);

    svg.selectAll('rect')
        .data(data)
        .enter()
        .append('rect')
        .attr('x', (d, i) => i * 30)
        .attr('y', d => height - d)
        .attr('width', 25)
        .attr('height', d => d)
        .style('fill', 'orange')
        .on('mouseover', function() {
            D3.select(this).style('fill', 'magenta');
        })
        .on('mouseout', function() {
            D3.select(this).style('fill', 'orange');
        });

    console.log('D3 Bar Chart drawn');

    // Create Three.js scene
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

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
        console.log('Animating cube');
    };

    animate();

    // Generate a Mermaid Sequence Diagram
    const graphDefinition = 'sequenceDiagram\nAlice->>John: Hello John, how are you?\nJohn-->>Alice: Great!';
    mermaid.mermaidAPI.render('graphDiv', graphDefinition, (svgCode) => {
        const graphDiv = document.createElement('div');
        graphDiv.innerHTML = svgCode;
        document.body.appendChild(graphDiv);
    });
    
    console.log('Mermaid chart generated');
});
