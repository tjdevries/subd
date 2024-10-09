const initPage = () => {
    const songs = document.querySelectorAll('.unplayed_song a');
    const images = document.querySelectorAll('.ai_song_image img');
    const videos = document.querySelectorAll('.video video');

    // Add an animated glowing effect to song titles
    songs.forEach(song => {
        song.addEventListener('mouseover', () => {
            song.style.transition = 'color 0.5s ease-in-out';
            song.style.color = 'yellow';
            song.style.textShadow = '0px 0px 10px yellow';
        });
        song.addEventListener('mouseout', () => {
            song.style.color = '';
            song.style.textShadow = '';
        });
    });

    // Add an animation to images when voting
    images.forEach(image => {
        image.addEventListener('click', () => {
            image.style.transition = 'transform 0.5s';
            image.style.transform = 'rotate(360deg) scale(1.1)';
            setTimeout(() => {
                image.style.transform = 'rotate(0deg) scale(1)';
            }, 500);
        });
    });

    // Loop through videos and animate their presence on the screen
    videos.forEach((video, index) => {
        setTimeout(() => {
            video.style.transition = 'opacity 1s';
            video.style.opacity = '1';
        }, index * 1000);
    });

    // Initialize Three.js for 3D animations
    const scene = new THREE.Scene();
    const camera = new THREE.PerspectiveCamera(75, window.innerWidth / window.innerHeight, 0.1, 1000);
    const renderer = new THREE.WebGLRenderer();
    renderer.setSize(window.innerWidth, window.innerHeight);
    document.body.appendChild(renderer.domElement);

    // Adding rotating cube
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
}

window.onload = initPage;