// =============================================================================
// File:           mobile/android/core/network/src/main/kotlin/com/qervon/core/network/QervonApiService.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Retrofit service interface: one method per HTTP endpoint this app
//   calls, matching `backend/apps/api-gateway/src/http.rs` route-for-route.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.network

import com.qervon.core.common.model.AppNotification
import com.qervon.core.common.model.Courier
import com.qervon.core.common.model.CourierWallet
import com.qervon.core.common.model.CustomerProfile
import com.qervon.core.common.model.CustomerRating
import com.qervon.core.common.model.FareQuote
import com.qervon.core.common.model.LocationSnapshot
import com.qervon.core.common.model.Order
import com.qervon.core.common.model.QervonUser
import com.qervon.core.common.model.SupportTicket
import okhttp3.MultipartBody
import okhttp3.ResponseBody
import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.DELETE
import retrofit2.http.GET
import retrofit2.http.Multipart
import retrofit2.http.POST
import retrofit2.http.Part
import retrofit2.http.Path
import retrofit2.http.Query

interface QervonApiService {
    // These two return only a status code with no JSON body — declaring
    // the target type as `ResponseBody` makes Retrofit skip the
    // kotlinx-serialization converter entirely (a well-defined core
    // Retrofit special case), rather than trying to parse an empty body.
    @POST("v1/auth/register")
    suspend fun register(@Body body: RegisterAccountBody): Response<ResponseBody>

    @POST("v1/auth/login")
    suspend fun login(@Body body: LoginBody): Response<AuthResponseBody>

    @POST("v1/auth/otp/request")
    suspend fun requestOtp(@Body body: OtpRequestBody): Response<OtpRequestResponseBody>

    @POST("v1/auth/otp/verify")
    suspend fun verifyOtp(@Body body: OtpVerifyBody): Response<AuthResponseBody>

    @POST("v1/auth/refresh")
    suspend fun refresh(@Body body: RefreshBody): Response<AuthResponseBody>

    @POST("v1/auth/phone")
    suspend fun setPhone(@Body body: SetPhoneBody): Response<QervonUser>

    // ---- Courier ----

    @GET("v1/courier/me")
    suspend fun getOwnCourier(): Response<Courier>

    @GET("v1/courier/me/wallet")
    suspend fun getOwnWallet(): Response<CourierWallet>

    @GET("v1/courier/me/ratings")
    suspend fun getOwnRatings(): Response<List<CustomerRating>>

    @POST("v1/courier/me/status")
    suspend fun setOwnAvailability(@Body body: SetCourierAvailabilityBody): Response<Courier>

    @POST("v1/courier/me/location")
    suspend fun updateOwnLocation(@Body body: UpdateLocationBody): Response<Courier>

    // Backend returns a bare JSON `null` when there is no pending offer.
    // Retrofit's converter picks a non-nullable serializer for a wrapped
    // `Response<T>` generic (Java type erasure drops the `?`), so the raw
    // body is decoded manually in QervonApi instead of relying on that.
    @GET("v1/courier/me/offer")
    suspend fun getOwnPendingOffer(): Response<ResponseBody>

    @GET("v1/courier/orders")
    suspend fun listCourierOrders(): Response<List<Order>>

    @POST("v1/courier/orders/{id}/accept")
    suspend fun acceptOffer(@Path("id") orderId: String): Response<Order>

    @POST("v1/courier/orders/{id}/reject")
    suspend fun rejectOffer(@Path("id") orderId: String): Response<ResponseBody>

    @POST("v1/courier/orders/{id}/pickup")
    suspend fun pickupOrder(@Path("id") orderId: String): Response<Order>

    @POST("v1/courier/orders/{id}/deliver")
    suspend fun deliverOrder(@Path("id") orderId: String, @Body body: CompleteDeliveryBody): Response<Order>

    @Multipart
    @POST("v1/courier/orders/{id}/photo-evidence")
    suspend fun uploadDeliveryPhoto(
        @Path("id") orderId: String,
        @Part photo: MultipartBody.Part,
    ): Response<UploadedFileResponseBody>

    // ---- Customer ----

    @GET("v1/customer/profile")
    suspend fun getCustomerProfile(): Response<CustomerProfile>

    @POST("v1/customer/profile/addresses")
    suspend fun addAddress(@Body body: CreateCustomerAddressBody): Response<CustomerProfile>

    @DELETE("v1/customer/profile/addresses/{id}")
    suspend fun removeAddress(@Path("id") addressId: String): Response<CustomerProfile>

    @GET("v1/customer/fare-quote")
    suspend fun getFareQuote(
        @Query("pickup_latitude") pickupLatitude: Double,
        @Query("pickup_longitude") pickupLongitude: Double,
        @Query("dropoff_latitude") dropoffLatitude: Double,
        @Query("dropoff_longitude") dropoffLongitude: Double,
    ): Response<FareQuote>

    @POST("v1/customer/orders")
    suspend fun createOrder(@Body body: CreateCustomerOrderBody): Response<Order>

    @GET("v1/customer/orders")
    suspend fun listCustomerOrders(): Response<List<Order>>

    @POST("v1/customer/orders/{id}/cancel")
    suspend fun cancelOrder(@Path("id") orderId: String): Response<Order>

    // See getOwnPendingOffer() note: null body decoded manually in QervonApi.
    @GET("v1/customer/orders/{id}/eta")
    suspend fun getOrderEta(@Path("id") orderId: String): Response<ResponseBody>

    @POST("v1/customer/orders/{id}/rating")
    suspend fun rateOrder(@Path("id") orderId: String, @Body body: RateOrderBody): Response<CustomerRating>

    @POST("v1/customer/support-tickets")
    suspend fun createSupportTicket(@Body body: OpenSupportTicketBody): Response<SupportTicket>

    @GET("v1/customer/support-tickets")
    suspend fun listSupportTickets(): Response<List<SupportTicket>>

    @GET("v1/customer/notifications")
    suspend fun listNotifications(): Response<List<AppNotification>>

    // ---- Shared ----

    @GET("v1/orders/{id}/tracking")
    suspend fun getOrderTracking(@Path("id") orderId: String): Response<LocationSnapshot>
}
