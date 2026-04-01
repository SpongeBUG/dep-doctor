import requests

# Vulnerable: Authorization header on a request that may redirect
def fetch_data(url):
    response = requests.get(url, headers={"Authorization": "Bearer token123"})
    return response.json()
