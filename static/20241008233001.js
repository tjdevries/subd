document.addEventListener('DOMContentLoaded', function() {
    // Add animation to header text
    const header = document.querySelector('.header a');
    header.style.animation = 'colorChange 5s infinite';

    // Add hover effect to navigation links
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
        link.addEventListener('mouseover', () => link.style.color = 'orange');
        link.addEventListener('mouseleave', () => link.style.color = '');
    });

    // Animate songs in the playlist
    const songs = document.querySelectorAll('.unplayed_song');
    songs.forEach((song, index) => {
        song.style.animation = `bounce 2s ${index * 0.5}s infinite alternate`;
    });

    // Add image interaction with voting mechanism
    const images = document.querySelectorAll('.ai_song_image img');
    images.forEach((img, index) => {
        img.addEventListener('click', () => alert(`Image ${index} clicked!`));
    });

    // Animate video elements
    const videos = document.querySelectorAll('.video video');
    videos.forEach(video => {
        video.style.filter = 'brightness(0.8)';
        video.style.transition = 'all 0.3s';
        video.addEventListener('mouseover', () => video.style.filter = 'brightness(1)');
        video.addEventListener('mouseleave', () => video.style.filter = 'brightness(0.8)');
    });

    // Animate audio controls
    const audioPlayers = document.querySelectorAll('audio');
    audioPlayers.forEach(audio => {
        audio.style.border = '2px solid #000';
        audio.style.boxShadow = '0 0 10px rgba(0, 0, 0, 0.5)';
    });

});