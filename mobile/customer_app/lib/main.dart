// =============================================================================
// File:           mobile/customer_app/lib/main.dart
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Version:        0.1.0
// Description:    Qervon Mobile Customer App Main Entry & Order Flow
// =============================================================================

import 'package:flutter/material.dart';

void main() {
  runApp(const QervonCustomerApp());
}

class QervonCustomerApp extends StatelessWidget {
  const QervonCustomerApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Qervon Customer',
      debugShowCheckedModeBanner: false,
      theme: ThemeData.dark().copyWith(
        scaffoldBackgroundColor: const Color(0xFF060913),
        colorScheme: const ColorScheme.dark(
          primary: Color(0xFF10B981),
          secondary: Color(0xFF38BDF8),
        ),
      ),
      home: const CustomerHomeScreen(),
    );
  }
}

class CustomerHomeScreen extends StatefulWidget {
  const CustomerHomeScreen({super.key});

  @override
  State<CustomerHomeScreen> createState() => _CustomerHomeScreenState();
}

class _CustomerHomeScreenState extends State<CustomerHomeScreen> {
  int _selectedIndex = 0;
  String _selectedPackageType = 'Zarf / Evrak';
  double _estimatedFare = 45.0;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        backgroundColor: const Color(0xFF0F172A),
        title: Row(
          children: [
            Container(
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: const Color(0xFF10B981),
                borderRadius: BorderRadius.circular(8),
              ),
              child: const Icon(Icons.delivery_dining, color: Colors.white),
            ),
            const SizedBox(width: 12),
            const Text(
              'QERVON CUSTOMER',
              style: TextStyle(fontWeight: FontWeight.bold, fontSize: 16),
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.notifications_none),
            onPressed: () {},
          ),
        ],
      ),
      body: IndexedStack(
        index: _selectedIndex,
        children: [
          _buildOrderCourierTab(),
          _buildActiveTrackingTab(),
          _buildOrderHistoryTab(),
        ],
      ),
      bottomNavigationBar: BottomNavigationBar(
        currentIndex: _selectedIndex,
        onTap: (index) => setState(() => _selectedIndex = index),
        backgroundColor: const Color(0xFF0F172A),
        selectedItemColor: const Color(0xFF10B981),
        unselectedItemColor: Colors.grey,
        items: const [
          BottomNavigationBarItem(
            icon: Icon(Icons.add_location_alt),
            label: 'Kurye Çağır',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.map),
            label: 'Canlı Takip',
          ),
          BottomNavigationBarItem(
            icon: Icon(Icons.history),
            label: 'Siparişlerim',
          ),
        ],
      ),
    );
  }

  Widget _buildOrderCourierTab() {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(16.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Banner
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              gradient: const LinearGradient(
                colors: [Color(0xFF0284C7), Color(0xFF9333EA)],
              ),
              borderRadius: BorderRadius.circular(16),
            ),
            child: Row(
              children: [
                const Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        '⚡ 30 Dakikada VIP Kurye Kapında',
                        style: TextStyle(
                            fontSize: 16, fontWeight: FontWeight.bold),
                      ),
                      SizedBox(height: 4),
                      Text(
                        'AI Dispatcher ile en yakın motorlu kurye atanır.',
                        style: TextStyle(fontSize: 12, color: Colors.white70),
                      ),
                    ],
                  ),
                ),
                ElevatedButton(
                  onPressed: () {},
                  style: ElevatedButton.styleFrom(
                    backgroundColor: Colors.white,
                    foregroundColor: Colors.black,
                  ),
                  child: const Text('Tıkla Çağır'),
                ),
              ],
            ),
          ),
          const SizedBox(height: 24),
          const Text(
            'Alım & Teslimat Noktaları',
            style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 12),
          TextField(
            decoration: InputDecoration(
              prefixIcon: const Icon(Icons.circle, color: Color(0xFF10B981)),
              hintText: 'Nereden Alınacak? (Örn: Sultanahmet Mah.)',
              filled: true,
              fillColor: const Color(0xFF0F172A),
              border:
                  OutlineInputBorder(borderRadius: BorderRadius.circular(12)),
            ),
          ),
          const SizedBox(height: 12),
          TextField(
            decoration: InputDecoration(
              prefixIcon: const Icon(Icons.location_on, color: Colors.redAccent),
              hintText: 'Nereye Teslim Edilecek? (Örn: Maslak Plaza)',
              filled: true,
              fillColor: const Color(0xFF0F172A),
              border:
                  OutlineInputBorder(borderRadius: BorderRadius.circular(12)),
            ),
          ),
          const SizedBox(height: 24),
          const Text(
            'Paket Tipi Seçin',
            style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 12),
          Row(
            children: [
              _packageTypeCard('Zarf / Evrak', Icons.description),
              const SizedBox(width: 8),
              _packageTypeCard('Gıda Paket', Icons.fastfood),
              const SizedBox(width: 8),
              _packageTypeCard('Koli (VIP)', Icons.inventory_2),
            ],
          ),
          const SizedBox(height: 24),
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: const Color(0xFF0F172A),
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: Colors.white12),
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text('Tahmini Taşıma Ücreti:'),
                Text(
                  '₺ ${_estimatedFare.toStringAsFixed(2)}',
                  style: const TextStyle(
                      fontSize: 20,
                      fontWeight: FontWeight.bold,
                      color: Color(0xFF10B981)),
                ),
              ],
            ),
          ),
          const SizedBox(height: 24),
          SizedBox(
            width: double.infinity,
            height: 52,
            child: ElevatedButton(
              onPressed: () {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(
                    content: Text('🎉 Kurye İsteğiniz Alındı! AI Atama Yapıldı.'),
                  ),
                );
                setState(() => _selectedIndex = 1);
              },
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFF10B981),
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(12),
                ),
              ),
              child: const Text(
                'KURYEYİ HIZLICA ÇAĞIR',
                style: TextStyle(
                    fontSize: 16,
                    fontWeight: FontWeight.bold,
                    color: Colors.white),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _packageTypeCard(String label, IconData icon) {
    final isSelected = _selectedPackageType == label;
    return Expanded(
      child: GestureDetector(
        onTap: () => setState(() => _selectedPackageType = label),
        child: Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: isSelected
                ? const Color(0xFF10B981).withOpacity(0.2)
                : const Color(0xFF0F172A),
            border: Border.all(
              color: isSelected ? const Color(0xFF10B981) : Colors.white12,
            ),
            borderRadius: BorderRadius.circular(12),
          ),
          child: Column(
            children: [
              Icon(icon, color: isSelected ? const Color(0xFF10B981) : Colors.grey),
              const SizedBox(height: 8),
              Text(
                label,
                style: TextStyle(
                  fontSize: 11,
                  fontWeight: FontWeight.bold,
                  color: isSelected ? Colors.white : Colors.grey,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildActiveTrackingTab() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Container(
            padding: const EdgeInsets.all(24),
            decoration: BoxDecoration(
              color: const Color(0xFF10B981).withOpacity(0.1),
              shape: BoxShape.circle,
            ),
            child: const Icon(Icons.two_wheeler, size: 64, color: Color(0xFF10B981)),
          ),
          const SizedBox(height: 16),
          const Text(
            'Ahmet Kurye Yolda! 🛵',
            style: TextStyle(fontSize: 20, fontWeight: FontWeight.bold),
          ),
          const SizedBox(height: 8),
          const Text(
            'Tahmini Varış (ETA): 11 Dakika',
            style: TextStyle(color: Colors.grey),
          ),
        ],
      ),
    );
  }

  Widget _buildOrderHistoryTab() {
    return ListView.builder(
      padding: const EdgeInsets.all(16),
      itemCount: 2,
      itemBuilder: (context, index) {
        return Card(
          color: const Color(0xFF0F172A),
          margin: const EdgeInsets.only(bottom: 12),
          child: ListTile(
            leading: const CircleAvatar(
              backgroundColor: Color(0xFF10B981),
              child: Icon(Icons.check, color: Colors.white),
            ),
            title: Text('Sipariş #${99180 + index}'),
            subtitle: const Text('Sultanahmet ➔ Maslak Plaza'),
            trailing: const Text('₺ 45.00', style: TextStyle(fontWeight: FontWeight.bold)),
          ),
        );
      },
    );
  }
}
