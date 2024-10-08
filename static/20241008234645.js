// Include Three.js and additional libraries like GSAP to animate elements
import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import gsap from 'gsap';

// Initialize the scene
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(75, window.innerWidth/window.innerHeight, 0.1, 1000);
const renderer = new THREE.WebGLRenderer();
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Add lighting
const ambientLight = new THREE.AmbientLight(0x404040); // soft white light
scene.add(ambientLight);
const pointLight = new THREE.PointLight(0xffffff, 1, 100);
pointLight.position.set(50, 50, 50);
scene.add(pointLight);

// Load a futuristic 3D model 
const loader = new GLTFLoader();
loader.load('/path/to/model.glb', function(gltf) {
    scene.add(gltf.scene);

    // Animate the 3D model
    gsap.to(gltf.scene.rotation, { y: Math.PI * 2, duration: 20, repeat: -1, ease: 'none' });
}, undefined, function(error) {
    console.error(error);
});

// Animate the header using GSAP for a neon glow effect
const header = document.querySelector('.header');
gsap.fromTo(header, { textShadow: '0 0 0px rgba(255, 255, 255, 0.2)' }, 
{ textShadow: '0 0 20px rgba(255, 0, 255, 1)', duration: 1, repeat: -1, yoyo: true });

// Create an animating starfield background
const starsGeometry = new THREE.Geometry();
for (let i = 0; i < 10000; i++) {
    const star = new THREE.Vector3(
        THREE.Math.randFloatSpread(2000),
        THREE.Math.randFloatSpread(2000),
        THREE.Math.randFloatSpread(2000)
    );
    starsGeometry.vertices.push(star);
}

const starsMaterial = new THREE.PointsMaterial({ color: 0xaaaaaa });
const starField = new THREE.Points(starsGeometry, starsMaterial);
scene.add(starField);

// Render the scene
camera.position.z = 5;
const animate = function() {
    requestAnimationFrame(animate);

    // Rotate the starfield
    starField.rotation.x += 0.001;
    starField.rotation.y += 0.001;

    renderer.render(scene, camera);
};
animate();