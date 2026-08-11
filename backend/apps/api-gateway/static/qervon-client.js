/* Qervon Browser SDK v0.1 — session-cookie based client. */
export class QervonClient {
  constructor(baseUrl = '') { this.baseUrl = baseUrl.replace(/\/$/, ''); }
  csrfHeaders() {
    const value = document.cookie.split('; ').find(v => v.startsWith('qervon_csrf_token='))?.split('=')[1] || '';
    return { 'content-type': 'application/json', 'x-csrf-token': value };
  }
  async request(path, options = {}) {
    const response = await fetch(`${this.baseUrl}${path}`, { credentials: 'same-origin', ...options, headers: { ...this.csrfHeaders(), ...(options.headers || {}) } });
    if (!response.ok) throw new Error(`Qervon API ${response.status}`);
    return response.status === 204 ? null : response.json();
  }
  createCustomerOrder(order) { return this.request('/v1/customer/orders', { method: 'POST', body: JSON.stringify(order) }); }
  customerOrders() { return this.request('/v1/customer/orders'); }
  courierOrders() { return this.request('/v1/courier/orders'); }
  courierPickup(id) { return this.request(`/v1/courier/orders/${id}/pickup`, { method: 'POST' }); }
  courierDeliver(id, proof) { return this.request(`/v1/courier/orders/${id}/deliver`, { method: 'POST', body: JSON.stringify(proof) }); }
  operationsOverview() { return this.request('/v1/operations/overview'); }
}
