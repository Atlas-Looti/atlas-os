"use client";

import { useMemo } from "react";
import { useAuth } from "@clerk/nextjs";
import { useQuery } from "@tanstack/react-query";
import { BarChart, Bar, CartesianGrid, XAxis, YAxis } from "recharts";
import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
} from "@/components/ui/card";
import {
    ChartContainer,
    ChartTooltip,
    ChartTooltipContent,
    type ChartConfig,
} from "@/components/ui/chart";

const API_BASE = process.env["NEXT_PUBLIC_API_URL"] ?? "http://localhost:3001";

interface UsageRecord {
    id: string;
    action: string;
    workflow: string | null;
    duration_ms: number | null;
    status: "success" | "error" | "pending";
    error_msg: string | null;
    metadata: Record<string, unknown>;
    created_at: string;
}

interface UsageResponse {
    data: UsageRecord[];
    meta: { total: number; limit: number; offset: number };
}

const chartConfig = {
    count: { label: "Events", color: "hsl(var(--chart-1))" },
    success: { label: "Success", color: "hsl(var(--chart-2))" },
    error: { label: "Error", color: "hsl(var(--chart-5))" },
} satisfies ChartConfig;

export function ComputeUsageChart() {
    const { getToken } = useAuth();

    const { data, isLoading, isError } = useQuery<UsageResponse>({
        queryKey: ["computeUsageDashboard"],
        queryFn: async () => {
            const token = await getToken();
            if (!token) throw new Error("Not authenticated");
            const res = await fetch(`${API_BASE}/keys/compute-usage?limit=500`, {
                headers: { Authorization: `Bearer ${token}` },
            });
            if (!res.ok) throw new Error("Failed to fetch usage");
            return res.json() as Promise<UsageResponse>;
        },
        refetchInterval: 30_000,
    });

    const chartData = useMemo(() => {
        if (!data?.data.length) return [];
        const agg: Record<string, { action: string; success: number; error: number }> = {};
        for (const row of data.data) {
            if (!agg[row.action]) agg[row.action] = { action: row.action, success: 0, error: 0 };
            if (row.status === "success") agg[row.action]!.success++;
            else if (row.status === "error") agg[row.action]!.error++;
        }
        return Object.values(agg).sort((a, b) => (b.success + b.error) - (a.success + a.error));
    }, [data]);

    const timelineData = useMemo(() => {
        const days: Record<string, { date: string; count: number }> = {};
        const now = Date.now();
        for (let i = 6; i >= 0; i--) {
            const d = new Date(now - i * 86_400_000);
            const key = d.toLocaleDateString("en-US", { month: "short", day: "numeric" });
            days[key] = { date: key, count: 0 };
        }
        if (data?.data) {
            for (const row of data.data) {
                const key = new Date(row.created_at).toLocaleDateString("en-US", { month: "short", day: "numeric" });
                if (days[key]) days[key]!.count++;
            }
        }
        return Object.values(days);
    }, [data]);

    const total = data?.meta.total ?? 0;
    const successCount = data?.data.filter((r) => r.status === "success").length ?? 0;
    const errorCount = data?.data.filter((r) => r.status === "error").length ?? 0;

    if (isLoading) {
        return (
            <div className="space-y-3">
                <div className="h-5 w-40 rounded-md bg-muted animate-pulse" />
                <div className="h-40 rounded-xl bg-muted animate-pulse" />
            </div>
        );
    }

    if (isError) {
        return (
            <Card>
                <CardContent className="pt-6 text-center text-sm text-muted-foreground">
                    Could not load compute usage.
                </CardContent>
            </Card>
        );
    }

    return (
        <div className="space-y-4">
            <div>
                <h2 className="text-xl font-semibold tracking-tight">Compute Usage</h2>
                <p className="text-sm text-muted-foreground">
                    CLI &amp; workflow events posted by the Atlas OS platform.
                </p>
            </div>

            {total === 0 ? (
                <Card>
                    <CardContent className="pt-6 text-center text-sm text-muted-foreground">
                        No events recorded yet.
                    </CardContent>
                </Card>
            ) : (
                <>
                    {/* Stat cards */}
                    <div className="grid grid-cols-3 gap-3">
                        {[
                            { label: "Total Events", value: total },
                            { label: "Success", value: successCount },
                            { label: "Errors", value: errorCount },
                        ].map((s) => (
                            <Card key={s.label}>
                                <CardHeader className="pb-1 pt-4 px-4">
                                    <CardDescription>{s.label}</CardDescription>
                                </CardHeader>
                                <CardContent className="pb-4 px-4">
                                    <p className="text-2xl font-semibold">{s.value}</p>
                                </CardContent>
                            </Card>
                        ))}
                    </div>

                    {/* Last 7 days */}
                    <Card>
                        <CardHeader>
                            <CardTitle className="text-sm font-medium">Last 7 Days</CardTitle>
                        </CardHeader>
                        <CardContent>
                            <ChartContainer config={chartConfig} className="min-h-[140px] w-full">
                                <BarChart data={timelineData} accessibilityLayer>
                                    <CartesianGrid vertical={false} />
                                    <XAxis
                                        dataKey="date"
                                        tickLine={false}
                                        axisLine={false}
                                        tickMargin={8}
                                        tick={{ fontSize: 11 }}
                                    />
                                    <ChartTooltip content={<ChartTooltipContent hideIndicator />} />
                                    <Bar dataKey="count" fill="var(--color-count)" radius={4} name="Events" />
                                </BarChart>
                            </ChartContainer>
                        </CardContent>
                    </Card>

                    {/* By action */}
                    {chartData.length > 0 && (
                        <Card>
                            <CardHeader>
                                <CardTitle className="text-sm font-medium">By Action</CardTitle>
                            </CardHeader>
                            <CardContent>
                                <ChartContainer config={chartConfig} className="min-h-[140px] w-full">
                                    <BarChart data={chartData} layout="vertical" accessibilityLayer>
                                        <CartesianGrid horizontal={false} />
                                        <XAxis type="number" tickLine={false} axisLine={false} tick={{ fontSize: 11 }} />
                                        <YAxis
                                            type="category"
                                            dataKey="action"
                                            tickLine={false}
                                            axisLine={false}
                                            tick={{ fontSize: 11 }}
                                            width={110}
                                        />
                                        <ChartTooltip content={<ChartTooltipContent />} />
                                        <Bar dataKey="success" stackId="a" fill="var(--color-success)" name="Success" />
                                        <Bar dataKey="error" stackId="a" fill="var(--color-error)" radius={[0, 4, 4, 0]} name="Error" />
                                    </BarChart>
                                </ChartContainer>
                            </CardContent>
                        </Card>
                    )}
                </>
            )}
        </div>
    );
}
