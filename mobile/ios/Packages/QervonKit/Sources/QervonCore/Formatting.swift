// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Formatting.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Shared date/time display formatting used across feature screens.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public enum QervonFormat {
    public static func relativeTime(_ date: Date, now: Date = Date()) -> String {
        let formatter = RelativeDateTimeFormatter()
        formatter.locale = Locale(identifier: "tr_TR")
        formatter.unitsStyle = .short
        return formatter.localizedString(for: date, relativeTo: now)
    }

    public static func time(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "tr_TR")
        formatter.dateFormat = "HH:mm"
        return formatter.string(from: date)
    }

    public static func dayAndTime(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "tr_TR")
        formatter.dateFormat = "d MMM, HH:mm"
        return formatter.string(from: date)
    }

    /// Midnight (device-local) of the day containing `date`.
    public static func startOfDay(containing date: Date = Date()) -> Date {
        Calendar.current.startOfDay(for: date)
    }

    public static func startOfWeek(containing date: Date = Date()) -> Date {
        var calendar = Calendar.current
        calendar.firstWeekday = 2 // Monday
        let components = calendar.dateComponents([.yearForWeekOfYear, .weekOfYear], from: date)
        return calendar.date(from: components) ?? startOfDay(containing: date)
    }

    public static func startOfMonth(containing date: Date = Date()) -> Date {
        let calendar = Calendar.current
        let components = calendar.dateComponents([.year, .month], from: date)
        return calendar.date(from: components) ?? startOfDay(containing: date)
    }
}
