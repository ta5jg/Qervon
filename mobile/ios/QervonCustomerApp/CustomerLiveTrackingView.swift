import SwiftUI
import QervonCore
import QervonNetworking
import QervonDesignSystem
import CustomerOrderFeature

struct CustomerLiveTrackingView: View {
    @StateObject private var viewModel: OrderHistoryViewModel

    init(api: QervonAPI) {
        _viewModel = StateObject(wrappedValue: OrderHistoryViewModel(api: api))
    }

    var body: some View {
        ScrollView {
            VStack(spacing: QervonSpacing.md) {
                QervonCard {
                    VStack(alignment: .leading, spacing: QervonSpacing.sm) {
                        RoundedRectangle(cornerRadius: 16)
                            .fill(QervonColor.surface)
                            .overlay(
                                Image(systemName: "map.fill")
                                    .font(.system(size: 28, weight: .bold))
                                    .foregroundColor(QervonColor.cyan)
                            )
                            .frame(height: 220)

                        Text("ATANAN SÜRÜCÜ")
                            .font(.system(size: 11, weight: .bold))
                            .foregroundColor(QervonColor.textSecondary)
                        Text(activeOrder == nil ? "Aktif sipariş bekleniyor" : "Atanan kurye canlı takipte")
                            .font(.system(size: 15, weight: .bold))
                            .foregroundColor(QervonColor.success)
                        Text(activeOrder == nil ? "Kurye ataması bekleniyor" : "TAHMİNİ ETA: 3 Dk")
                            .font(.system(size: 12, weight: .semibold))
                            .foregroundColor(QervonColor.textSecondary)
                        Button("BİLDİRİMLERİ AÇ") {}
                            .buttonStyle(QervonButtonStyle(kind: .secondary))
                    }
                }

                QervonCard {
                    VStack(alignment: .leading, spacing: QervonSpacing.xs) {
                        Text("Hızlı Kurye Çağır")
                            .font(.system(size: 14, weight: .bold))
                            .foregroundColor(QervonColor.textPrimary)
                        Text("Yeni siparişleri Sipariş Ver sekmesinden oluşturabilirsiniz.")
                            .font(.system(size: 12))
                            .foregroundColor(QervonColor.textSecondary)
                    }
                }
            }
            .padding(.horizontal, QervonSpacing.lg)
            .padding(.vertical, QervonSpacing.lg)
        }
        .qervonScreenBackground()
        .navigationTitle("Canlı Takip")
        .task { await viewModel.load() }
        .onAppear { viewModel.startLiveUpdates() }
        .onDisappear { viewModel.stopLiveUpdates() }
    }

    private var activeOrder: Order? {
        viewModel.activeOrders.first
    }
}
