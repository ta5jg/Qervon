// =============================================================================
// File:           mobile/courier_app/lib/main.dart
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Version:        0.1.0
// Description:    Qervon Mobile Courier App (Online/Offline, Accept Job, GPS, Sign)
// =============================================================================

import 'package:flutter/material.dart';

void main() {
  runApp(const QervonCourierApp());
}

class QervonCourierApp extends StatelessWidget {
  const QervonCourierApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Qervon Courier',
      debugShowCheckedModeBanner: false,
      theme: ThemeData.dark().copyWith(
        scaffoldBackgroundColor: const Color(0xFF060913),
        colorScheme: const ColorScheme.dark(
          primary: Color(0xFF38BDF8),
          secondary: Color(0xFFA855F7),
        ),
      ),
      home: const CourierHomeScreen(),
    );
  }
}

class CourierHomeScreen extends StatefulWidget {
  const CourierHomeScreen({super.key});

  @override
  State<CourierHomeScreen> createState() => _CourierHomeScreenState();
}

class _CourierHomeScreenState extends State<CourierHomeScreen> {
  bool _isOnline = true;
  bool _hasActiveJob = true;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: const Color(0xFF0F172A),
        title: const Text('QERVON KURYE MOBİL'),
        actions: [
          Switch(
            value: _isOnline,
            activeColor: const Color(0xFF10B981),
            onChanged: (val) => setState(() => _isOnline = val),
          ),
          const SizedBox(width: 8),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          children: [
            // Status Card
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: const Color(0xFF0F172A),
                borderRadius: BorderRadius.circular(16),
                border: Border.all(
                  color: _isOnline ? const Color(0xFF10B981) : Colors.grey,
                ),
              ),
              child: Row(
                children: [
                  CircleAvatar(
                    backgroundColor: _isOnline ? const Color(0xFF10B981) : Colors.grey,
                    child: Icon(_isOnline ? Icons.power_settings_new : Icons.power_off, color: Colors.white),
                  ),
                  const SizedBox(width: 16),
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        _isOnline ? 'ONLINE (Göreve Hazır)' : 'OFFLINE',
                        style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 16),
                      ),
                      Text(
                        _isOnline ? 'GPS Konum Yayını Aktif 📡' : 'GPS Yayını Durduruldu',
                        style: const TextStyle(color: Colors.grey, fontSize: 12),
                      ),
                    ],
                  ),
                ],
              ),
            ),
            const SizedBox(height: 24),

            // Active Job Card
            if (_hasActiveJob && _isOnline) ...[
              Container(
                padding: const EdgeInsets.all(20),
                decoration: BoxDecoration(
                  gradient: const LinearGradient(
                    colors: [Color(0xFF0F172A), Color(0xFF1E293B)],
                  ),
                  borderRadius: BorderRadius.circular(16),
                  border: Border.all(color: const Color(0xFF38BDF8)),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Container(
                          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                          decoration: BoxDecoration(
                            color: const Color(0xFFA855F7).withOpacity(0.2),
                            borderRadius: BorderRadius.circular(8),
                          ),
                          child: const Text('⚡ AI DISPATCH ATANDI', style: TextStyle(color: Color(0xFFA855F7), fontWeight: FontWeight.bold, fontSize: 11)),
                        ),
                        const Text('₺ 45.00', style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold, color: Color(0xFF10B981))),
                      ],
                    ),
                    const SizedBox(height: 16),
                    const Row(
                      children: [
                        Icon(Icons.circle, color: Color(0xFF10B981), size: 14),
                        SizedBox(width: 8),
                        Text('Alım: Sultanahmet Restoran'),
                      ],
                    ),
                    const SizedBox(height: 8),
                    const Row(
                      children: [
                        Icon(Icons.location_on, color: Colors.redAccent, size: 14),
                        SizedBox(width: 8),
                        Text('Teslim: Maslak Plaza Kat 8'),
                      ],
                    ),
                    const SizedBox(height: 20),
                    Row(
                      children: [
                        Expanded(
                          child: ElevatedButton.icon(
                            onPressed: () {
                              ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('Sipariş Teslim Edildi & Dijital İmza Alındı!')));
                              setState(() => _hasActiveJob = false);
                            },
                            icon: const Icon(Icons.check_circle),
                            label: const Text('TESLİM ET & İMZA AL'),
                            style: ElevatedButton.styleFrom(backgroundColor: const Color(0xFF10B981)),
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
            ] else ...[
              const Expanded(
                child: Center(
                  child: Text('Yeni İş Bildirimi Bekleniyor... 🚴‍♂️', style: TextStyle(color: Colors.grey)),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
