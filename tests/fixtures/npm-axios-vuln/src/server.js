const axios = require('axios');
const express = require('express');
const app = express();

// Vulnerable: passing req.query directly to axios
app.get('/proxy', async (req, res) => {
  const result = await axios.get(req.query.url);
  res.json(result.data);
});

// Vulnerable: axios.create with withCredentials: true
const apiClient = axios.create({
  baseURL: 'https://api.example.com',
  withCredentials: true,
});

app.listen(3000);
