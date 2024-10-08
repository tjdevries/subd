document.addEventListener('DOMContentLoaded', function() {
    // Add some fun animations to the header
    const header = document.querySelector('.header-container');
    header.style.transition = 'transform 0.6s ease-in-out';
    header.addEventListener('mouseenter', function() {
        header.style.transform = 'scale(1.1) rotate(3deg)';
    });
    header.addEventListener('mouseleave', function() {
        header.style.transform = 'scale(1) rotate(0deg)';
    });

    // Animate navigation links
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
        link.style.transition = 'color 0.3s ease-in-out';
        link.addEventListener('mouseenter', function() {
            link.style.color = 'orange';
        });
        link.addEventListener('mouseleave', function() {
            link.style.color = '';
        });
    });

    // Animate song items in the playlist
    const songItems = document.querySelectorAll('.unplayed_song');
    songItems.forEach(item => {
        item.style.transition = 'background-color 0.5s';
        item.addEventListener('mouseenter', function() {
            item.style.backgroundColor = '#f0f0f0';
        });
        item.addEventListener('mouseleave', function() {
            item.style.backgroundColor = '';
        });
    });

    // Add hover effects to the song cards
    const songs = document.querySelectorAll('.song');
    songs.forEach(song => {
        song.style.transition = 'transform 0.7s ease-in-out';
        song.addEventListener('mouseenter', function() {
            song.style.transform = 'scale(1.05)';
        });
        song.addEventListener('mouseleave', function() {
            song.style.transform = 'scale(1)';
        });
    });

    // Animate user links
    const userLinks = document.querySelectorAll('.username a');
    userLinks.forEach(user => {
        user.style.transition = 'color 0.5s ease-in-out';
        user.addEventListener('mouseenter', function() {
            user.style.color = '#ff6600';
        });
        user.addEventListener('mouseleave', function() {
            user.style.color = '';
        });
    });

    // Add fun animations to video containers
    const videos = document.querySelectorAll('.video');
    videos.forEach(video => {
        video.style.transition = 'transform 0.7s ease-in-out';
        video.addEventListener('mouseenter', function() {
            video.style.transform = 'scale(1.08)';
        });
        video.addEventListener('mouseleave', function() {
            video.style.transform = 'scale(1)';
        });
    });

    // Add animation to song containers in charts
    const songContainers = document.querySelectorAll('.song-container');
    songContainers.forEach(container => {
        container.style.transition = 'box-shadow 0.4s ease-in-out';
        container.addEventListener('mouseenter', function() {
            container.style.boxShadow = '0 4px 8px rgba(0, 0, 0, 0.2)';
        });
        container.addEventListener('mouseleave', function() {
            container.style.boxShadow = '';
        });
    });
});