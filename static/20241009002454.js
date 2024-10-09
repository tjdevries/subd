// JavaScript with animations using libraries such as Three.js and Anime.js
// Assumes Three.js and Anime.js are loaded in the HTML

// Setup Three.js scene
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

function animate() {
    requestAnimationFrame(animate);

    // Rotation animation
    cube.rotation.x += 0.01;
    cube.rotation.y += 0.01;

    // Add more complex animations using Anime.js
    anime({
        targets: cube.scale,
        x: [1, 2],
        y: [1, 2],
        z: [1, 2],
        duration: 1000,
        easing: 'easeInOutElastic',
        direction: 'alternate',
        loop: true
    });

    renderer.render(scene, camera);
}
animate();

// Fun hover animation
const navLinks = document.querySelectorAll('.nav-link');
navLinks.forEach(link => {
    link.addEventListener('mouseenter', () => {
        anime({
            targets: link,
            scale: 1.2,
            duration: 300,
            easing: 'easeInOutQuad'
        });
    });
    link.addEventListener('mouseleave', () => {
        anime({
            targets: link,
            scale: 1.0,
            duration: 300,
            easing: 'easeInOutQuad'
        });
    });
});