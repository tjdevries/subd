// Load required libraries for 3D animations
import * as THREE from 'three';

// Scene Setup
const scene = new THREE.Scene();

// Camera Setup
const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);

// Renderer Setup
const renderer = new THREE.WebGLRenderer({ antialias: true });
renderer.setSize(window.innerWidth, window.innerHeight);
document.body.appendChild(renderer.domElement);

// Light Setup
const ambientLight = new THREE.AmbientLight(0x404040, 5); 
scene.add(ambientLight);

// Snake Geometry (Inspired by Quetzalcoatl)
const snakeMaterial = new THREE.MeshStandardMaterial({ color: 0x556b2f });
const snakeSegment = new THREE.BoxGeometry(1, 0.4, 0.4);

const snake = new THREE.Group();
const segmentCount = 10;
for (let i = 0; i < segmentCount; i++) {
  const segment = new THREE.Mesh(snakeSegment, snakeMaterial);
  segment.position.y = i * 0.5;
  snake.add(segment);
}
scene.add(snake);

// Move the camera back so we can see the scene
camera.position.z = 10;

// Animation loop
function animate() {
  requestAnimationFrame(animate);
  // Rotate the snake
  snake.rotation.x += 0.01;
  snake.rotation.y += 0.01;
  // Render the scene
  renderer.render(scene, camera);
}

// Handle window resize
window.addEventListener('resize', () => {
  const width = window.innerWidth;
  const height = window.innerHeight;
  renderer.setSize(width, height);
  camera.aspect = width / height;
  camera.updateProjectionMatrix();
});

// Start the animation
animate();