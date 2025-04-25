// Simplified backend file for the casino project (based on your earlier provided code)
const express = require('express');
const app = express();
app.get('/api/health', (req, res) => res.json({ status: 'ok' }));
app.listen(3000, () => console.log('Backend running on port 3000'));