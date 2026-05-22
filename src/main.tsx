import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './index.css';

// Apply stored theme before first render to prevent flash. `themes.css` exposes
// "light" (Construction Amber on Deep Forge) and "openaec" (Dark Surface).
{
  const stored = localStorage.getItem('oes-theme') || 'openaec';
  document.documentElement.dataset.theme = stored === 'light' ? 'light' : 'openaec';
}

// Disable browser context menu in production
if (!import.meta.env.DEV) {
  document.addEventListener('contextmenu', (e) => e.preventDefault());
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
