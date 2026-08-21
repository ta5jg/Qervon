// =============================================================================
// File:           mobile/android/core/network/src/main/kotlin/com/qervon/core/network/QervonApi.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   High-level, typed façade over `QervonApiService` — every call either
//   returns the decoded body or throws a `QervonApiException`, so feature
//   ViewModels never touch Retrofit's `Response<T>` wrapper directly.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.network

import com.qervon.core.common.ApiErrorBody
import com.qervon.core.common.AuthTokenStore
import com.qervon.core.common.JsonConfig
import com.qervon.core.common.QervonApiException
import com.qervon.core.common.model.Address
import com.qervon.core.common.model.AppNotification
import com.qervon.core.common.model.AuthTokens
import com.qervon.core.common.model.Courier
import com.qervon.core.common.model.CourierWallet
import com.qervon.core.common.model.CustomerProfile
import com.qervon.core.common.model.CustomerRating
import com.qervon.core.common.model.EtaInfo
import com.qervon.core.common.model.FareQuote
import com.qervon.core.common.model.LocationSnapshot
import com.qervon.core.common.model.Order
import com.qervon.core.common.model.PendingOffer
import com.qervon.core.common.model.QervonUser
import com.qervon.core.common.model.SupportTicket
import kotlinx.serialization.SerializationException
import kotlinx.serialization.json.Json
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.MultipartBody
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.ResponseBody
import retrofit2.Response
import java.io.IOException

class QervonApi(
    private val service: QervonApiService,
    private val tokenStore: AuthTokenStore,
    private val json: Json = JsonConfig.shared,
) {
    // ---- Auth ----

    suspend fun register(email: String, displayName: String, password: String, tenantSlug: String?) {
        call { service.register(RegisterAccountBody(email, displayName, password, tenantSlug)) }
    }

    suspend fun login(email: String, password: String, tenantSlug: String): AuthTokens {
        val body = call { service.login(LoginBody(email, password, tenantSlug)) }
        return body.toDomain().also { tokenStore.save(it) }
    }

    suspend fun requestOtp(tenantSlug: String, phone: String): String? {
        val body = call { service.requestOtp(OtpRequestBody(tenantSlug, phone)) }
        return body.devCode
    }

    suspend fun verifyOtp(tenantSlug: String, phone: String, code: String): AuthTokens {
        val body = call { service.verifyOtp(OtpVerifyBody(tenantSlug, phone, code)) }
        return body.toDomain().also { tokenStore.save(it) }
    }

    suspend fun setPhone(phone: String): QervonUser = call { service.setPhone(SetPhoneBody(phone)) }

    fun logout() = tokenStore.clear()

    // ---- Courier ----

    suspend fun getOwnCourier(): Courier = call { service.getOwnCourier() }

    suspend fun getOwnWallet(): CourierWallet = call { service.getOwnWallet() }

    suspend fun getOwnRatings(): List<CustomerRating> = call { service.getOwnRatings() }

    suspend fun setOwnAvailability(online: Boolean): Courier =
        call { service.setOwnAvailability(SetCourierAvailabilityBody(online)) }

    suspend fun updateOwnLocation(latitude: Double, longitude: Double, speedKmh: Double?, batteryPct: Int?): Courier =
        call { service.updateOwnLocation(UpdateLocationBody(latitude, longitude, speedKmh, batteryPct)) }

    suspend fun getOwnPendingOffer(): PendingOffer? =
        fetchOptionalJson(PendingOffer.serializer()) { service.getOwnPendingOffer() }

    suspend fun listCourierOrders(): List<Order> = call { service.listCourierOrders() }

    suspend fun acceptOffer(orderId: String): Order = call { service.acceptOffer(orderId) }

    suspend fun rejectOffer(orderId: String) {
        call { service.rejectOffer(orderId) }
    }

    suspend fun pickupOrder(orderId: String, pickupPhotoEvidenceUrl: String): Order =
        call { service.pickupOrder(orderId, CompletePickupBody(pickupPhotoEvidenceUrl)) }

    suspend fun deliverOrder(
        orderId: String,
        recipientName: String,
        qrBarcodeVerified: Boolean,
        digitalSignatureBase64: String?,
        photoEvidenceUrl: String?,
        paymentCollected: Boolean,
    ): Order = call {
        service.deliverOrder(
            orderId,
            CompleteDeliveryBody(recipientName, qrBarcodeVerified, digitalSignatureBase64, photoEvidenceUrl, paymentCollected),
        )
    }

    /** Uploads a real delivery-proof photo (JPEG) for [orderId] to local
     * server-side storage and returns the URL to pass as
     * `photoEvidenceUrl` on [deliverOrder]. See
     * `backend/apps/api-gateway/src/http.rs`'s `upload_delivery_photo`. */
    suspend fun uploadOrderEvidencePhoto(orderId: String, jpegBytes: ByteArray): String {
        val body = jpegBytes.toRequestBody("image/jpeg".toMediaType())
        val part = MultipartBody.Part.createFormData("photo", "proof.jpg", body)
        return call { service.uploadDeliveryPhoto(orderId, part) }.url
    }

    suspend fun uploadDeliveryPhoto(orderId: String, jpegBytes: ByteArray): String =
        uploadOrderEvidencePhoto(orderId, jpegBytes)

    // ---- Customer ----

    suspend fun getCustomerProfile(): CustomerProfile = call { service.getCustomerProfile() }

    suspend fun addAddress(label: String, latitude: Double, longitude: Double, fullAddress: String): CustomerProfile =
        call { service.addAddress(CreateCustomerAddressBody(label, latitude, longitude, fullAddress)) }

    suspend fun removeAddress(addressId: String): CustomerProfile = call { service.removeAddress(addressId) }

    suspend fun getFareQuote(pickup: Address, dropoff: Address): FareQuote = call {
        service.getFareQuote(pickup.latitude, pickup.longitude, dropoff.latitude, dropoff.longitude)
    }

    suspend fun createOrder(
        pickup: Address,
        dropoff: Address,
        couponCode: String?,
        paymentMethod: String?,
        deliveryNote: String?,
        contactPhone: String?,
    ): Order = call {
        service.createOrder(CreateCustomerOrderBody(pickup, dropoff, couponCode, paymentMethod, deliveryNote, contactPhone))
    }

    suspend fun listCustomerOrders(): List<Order> = call { service.listCustomerOrders() }

    suspend fun cancelOrder(orderId: String): Order = call { service.cancelOrder(orderId) }

    /** Returns null when the order has no assigned courier yet or the
     * courier hasn't reported a location — an expected, non-error state
     * the backend surfaces as `Json<Option<EtaResponse>>`. */
    suspend fun getOrderEta(orderId: String): EtaInfo? =
        fetchOptionalJson(EtaInfo.serializer()) { service.getOrderEta(orderId) }

    suspend fun rateOrder(orderId: String, ratingStars: Int, comment: String?): CustomerRating =
        call { service.rateOrder(orderId, RateOrderBody(ratingStars, comment)) }

    suspend fun createSupportTicket(orderId: String?, subject: String, message: String): SupportTicket =
        call { service.createSupportTicket(OpenSupportTicketBody(orderId, subject, message)) }

    suspend fun listSupportTickets(): List<SupportTicket> = call { service.listSupportTickets() }

    suspend fun listNotifications(): List<AppNotification> = call { service.listNotifications() }

    // ---- Shared ----

    suspend fun getOrderTracking(orderId: String): LocationSnapshot = call { service.getOrderTracking(orderId) }

    // ---- Internals ----

    private fun AuthResponseBody.toDomain() = AuthTokens(accessToken, refreshToken, tokenType, expiresInSeconds.toInt())

    /** Reads a raw `Response<ResponseBody>`, treating a literal JSON
     * `null` (or an empty body) as "absent" and anything else as the
     * decoded [serializer] payload. Used for the handful of endpoints
     * that respond with `Json<Option<T>>` on the Rust side. */
    private suspend fun <T : Any> fetchOptionalJson(
        serializer: kotlinx.serialization.KSerializer<T>,
        request: suspend () -> Response<ResponseBody>,
    ): T? {
        val response = try {
            request()
        } catch (error: IOException) {
            throw QervonApiException.Transport(error)
        }
        if (!response.isSuccessful) {
            throw response.toApiException(json)
        }
        val text = response.body()?.string()?.trim()
        if (text.isNullOrEmpty() || text == "null") {
            return null
        }
        return try {
            json.decodeFromString(serializer, text)
        } catch (error: SerializationException) {
            throw QervonApiException.Decoding(error)
        }
    }

    private suspend inline fun <reified T> call(crossinline request: suspend () -> Response<T>): T {
        val response = try {
            request()
        } catch (error: IOException) {
            throw QervonApiException.Transport(error)
        }
        if (!response.isSuccessful) {
            throw response.toApiException(json)
        }
        val body = response.body()
        if (body == null) {
            @Suppress("UNCHECKED_CAST")
            return Unit as T
        }
        return body
    }
}

private fun <T> Response<T>.toApiException(json: Json): QervonApiException {
    val errorText = errorBody()?.string()
    if (errorText.isNullOrBlank()) {
        return QervonApiException.UnexpectedStatus(code())
    }
    return try {
        val decoded = json.decodeFromString(ApiErrorBody.serializer(), errorText)
        QervonApiException.Server(decoded.status, decoded.detail)
    } catch (_: SerializationException) {
        QervonApiException.UnexpectedStatus(code())
    }
}
