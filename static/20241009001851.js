document.addEventListener('DOMContentLoaded', function() {
    const header = document.querySelector('.header');
    const navLinks = document.querySelectorAll('.nav-link');
    const songLinks = document.querySelectorAll('.unplayed_song a, .chart-container a, .users-container a');
    const videoElements = document.querySelectorAll('video');
    // Animation Libraries
    // Initialize Three.js for 3D animations
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);
    const geometry = new THREE.IcosahedronGeometry(1, 0);
    const material = new THREE.MeshBasicMaterial({ color: 0x00ff00, wireframe: true });
    const icosahedron = new THREE.Mesh(geometry, material);
    scene.add(icosahedron);
    camera.position.z = 5;
    function animate() {
        requestAnimationFrame(animate);
        icosahedron.rotation.x += 0.01;
        icosahedron.rotation.y += 0.01;
        renderer.render(scene, camera);
    }
    animate();
    // Add some hover animations to song links
    songLinks.forEach(link => {
        link.addEventListener('mouseover', () => {
            link.style.color = 'yellow';
            link.style.transition = 'color 0.3s ease-in-out';
        });
        link.addEventListener('mouseout', () => {
            link.style.color = '';
        });
    });
    // Video auto play and loop
    videoElements.forEach(video => {
        video.addEventListener('mouseenter', () => {
            video.play();
        });
        video.addEventListener('mouseleave', () => {
            video.pause();
        });
    });
    // Background music for fun effect
    const audio = new Audio('/songs/background-music.mp3');
    audio.loop = true;
    audio.play();
    // Nav Link Hover Effect
    navLinks.forEach(link => {
        link.addEventListener('mouseover', () => {
            link.style.transform = 'scale(1.1)';
            link.style.transition = 'transform 0.3s ease';
        });
        link.addEventListener('mouseout', () => {
            link.style.transform = '';
        });
    });
});