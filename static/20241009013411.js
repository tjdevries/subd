// JavaScript to create a fun and animated page inspired by Blastoise

// Load Three.js scene
import * as THREE from 'three';

function createScene() {
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    const geometry = new THREE.SphereGeometry(1, 32, 32);
    const material = new THREE.MeshBasicMaterial({ color: 0x0077ff });
    const blastoiseShell = new THREE.Mesh(geometry, material);
    scene.add(blastoiseShell);

    camera.position.z = 5;

    function animate() {
        requestAnimationFrame(animate);
        blastoiseShell.rotation.x += 0.01;
        blastoiseShell.rotation.y += 0.01;
        renderer.render(scene, camera);
    }

    animate();
}

function splashAnimation() {
    // Simple splash effect using CSS animation
    const splash = document.createElement('div');
    splash.style.width = '100px';
    splash.style.height = '100px';
    splash.style.borderRadius = '50%';
    splash.style.background = 'blue';
    splash.style.position = 'absolute';
    splash.style.bottom = '-100px';
    splash.style.left = '50%';
    splash.style.animation = 'splash 1s ease-out infinite alternate';
    document.body.appendChild(splash);

    const styleSheet = document.styleSheets[0];
    styleSheet.insertRule(`
        @keyframes splash {
            0% { transform: scale(0.1); bottom: 0; }
            100% { transform: scale(1.2); bottom: 100%; }
        }
    `, styleSheet.cssRules.length);
}

function startFun() {
    console.log("Big Man Blastoise, Let the fun begin!");
    createScene();
    splashAnimation();
}

startFun();