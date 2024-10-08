document.addEventListener('DOMContentLoaded', () => {
    // Animate header
    const headerText = document.querySelector('.header a');
    headerText.style.animation = 'fadeIn 3s infinite alternate';

    // Header animation
    const style = document.createElement('style');
    style.innerHTML = `
    @keyframes fadeIn {
        from { opacity: 0; }
        to { opacity: 1; }
    }
    .nav-link {
        transition: color 0.3s, transform 0.3s;
    }
    .nav-link:hover {
        color: #ff5733;
        transform: translateY(-5px);
    }
    .unplayed_song a {
        display: block;
        padding: 5px;
        transition: background-color 0.3s;
    }
    .unplayed_song a:hover {
        background-color: #f0e68c;
        border-radius: 5px;
    }
    .current-song-info {
        animation: slideUp 2s ease-out;
    }
    @keyframes slideUp {
        from { transform: translateY(50px); opacity: 0; }
        to { transform: translateY(0); opacity: 1; }
    }
    .ai_song_image img {
        width: 100px;
        transition: transform 0.3s;
    }
    .ai_song_image img:hover {
        transform: scale(1.2);
    }
    .video video {
        border: 2px solid #ccc;
        border-radius: 10px;
        overflow: hidden;
    }
    `;
    document.head.append(style);

    // Fun animations
    const songsInPlaylist = document.querySelectorAll('.unplayed_song');
    songsInPlaylist.forEach((song, index) => {
        song.style.animation = `bounceIn ${(index + 1) * 0.5}s forwards`;
    });

    document.querySelector('.current-song').style.animation = 'highlight 3s infinite alternate';

    const highlightStyle = document.createElement('style');
    highlightStyle.innerHTML = `
    @keyframes bounceIn {
        from { transform: scale(0); opacity: 0; }
        to { transform: scale(1); opacity: 1; }
    }
    @keyframes highlight {
        from { background-color: #ffe4b5; }
        to { background-color: #ffa07a; }
    }
    `;
    document.head.append(highlightStyle);
});