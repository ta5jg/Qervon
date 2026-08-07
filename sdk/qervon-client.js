// =============================================================================
// File:           sdk/qervon-client.js
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Lightweight JavaScript SDK client for Qervon REST and Live WebSocket APIs.
//
// Specification:
//   QLS-000001 through QLS-000010.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

export class QervonClient {
  constructor(baseUrl = 'http://localhost:3000') {
    this.baseUrl = baseUrl.replace(/\/$/, '');
    this.wsUrl = this.baseUrl.replace(/^http/, 'ws') + '/ws/tracking';
  }

  // --- Users API ---
  async registerUser(userData) {
    const res = await fetch(`${this.baseUrl}/v1/users`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(userData),
    });
    if (!res.ok) throw new Error(`User registration failed: ${res.statusText}`);
    return res.json();
  }

  // --- Couriers API ---
  async registerCourier(courierData) {
    const res = await fetch(`${this.baseUrl}/v1/couriers`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(courierData),
    });
    if (!res.ok) throw new Error(`Courier registration failed: ${res.statusText}`);
    return res.json();
  }

  async listAvailableCouriers() {
    const res = await fetch(`${this.baseUrl}/v1/couriers`);
    if (!res.ok) throw new Error(`Listing couriers failed: ${res.statusText}`);
    return res.json();
  }

  async updateCourierLocation(courierId, latitude, longitude) {
    const res = await fetch(`${this.baseUrl}/v1/couriers/${courierId}/location`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ latitude, longitude }),
    });
    if (!res.ok) throw new Error(`Updating location failed: ${res.statusText}`);
    return res.json();
  }

  // --- Orders API ---
  async createOrder(orderData) {
    const res = await fetch(`${this.baseUrl}/v1/orders`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(orderData),
    });
    if (!res.ok) throw new Error(`Creating order failed: ${res.statusText}`);
    return res.json();
  }

  async assignCourier(orderId, courierId = null) {
    const res = await fetch(`${this.baseUrl}/v1/orders/${orderId}/assign`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ courier_id: courierId }),
    });
    if (!res.ok) throw new Error(`Assigning courier failed: ${res.statusText}`);
    return res.json();
  }

  // --- Live WebSocket Stream ---
  subscribeLiveTracking(onLocationUpdate) {
    const ws = new WebSocket(this.wsUrl);
    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        onLocationUpdate(data);
      } catch (err) {
        console.error('Failed to parse WS tracking message:', err);
      }
    };
    return ws;
  }
}
