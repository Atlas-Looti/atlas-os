"use client";

import { useAuth, useUser } from "@clerk/nextjs";
import { useState } from "react";
import { HugeiconsIcon } from "@hugeicons/react";
import {
    PlusSignIcon,
    Delete02Icon,
    Copy01Icon,
    Tick01Icon,
    Key01Icon,
    EyeIcon,
    ViewOffIcon,
} from "@hugeicons/core-free-icons";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { ComputeUsageChart } from "@/components/compute-usage-chart";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
    Card,
    CardContent,
} from "@/components/ui/card";

const API_BASE = process.env["NEXT_PUBLIC_API_URL"] ?? "http://localhost:3001";

interface ApiKey {
    id: string;
    user_id: string;
    name: string;
    prefix: string;
    created_at: string;
}

interface CreateKeyResponse {
    key: string;
    record: ApiKey;
}

function formatDate(iso: string) {
    return new Date(iso).toLocaleDateString("en-US", {
        year: "numeric",
        month: "short",
        day: "numeric",
    });
}

async function authHeaders(getToken: () => Promise<string | null>) {
    const token = await getToken();
    if (!token) throw new Error("Not authenticated");
    return {
        "Authorization": `Bearer ${token}`,
        "Content-Type": "application/json",
    } as const;
}

export default function DashboardPage() {
    const { user, isLoaded } = useUser();
    const { getToken } = useAuth();
    const queryClient = useQueryClient();

    const [newKeyName, setNewKeyName] = useState("");
    const [newlyCreatedKey, setNewlyCreatedKey] = useState<string | null>(null);
    const [copied, setCopied] = useState(false);
    const [keyVisible, setKeyVisible] = useState(false);
    const [deletingId, setDeletingId] = useState<string | null>(null);

    const { data: apiKeys = [], isLoading } = useQuery<ApiKey[]>({
        queryKey: ["apiKeys", user?.id],
        queryFn: async () => {
            const headers = await authHeaders(getToken);
            const res = await fetch(`${API_BASE}/keys`, { headers });
            if (!res.ok) throw new Error("Failed to fetch API keys");
            const data = await res.json() as { keys: ApiKey[] };
            return data.keys;
        },
        enabled: isLoaded && !!user,
    });

    const createMutation = useMutation({
        mutationFn: async (name: string) => {
            const headers = await authHeaders(getToken);
            const res = await fetch(`${API_BASE}/keys`, {
                method: "POST",
                headers,
                body: JSON.stringify({ name }),
            });
            if (!res.ok) throw new Error("Failed to create key");
            return res.json() as Promise<CreateKeyResponse>;
        },
        onSuccess: (data) => {
            queryClient.invalidateQueries({ queryKey: ["apiKeys", user?.id] });
            setNewlyCreatedKey(data.key);
            setKeyVisible(false);
            setNewKeyName("");
        },
    });

    const deleteMutation = useMutation({
        mutationFn: async (id: string) => {
            const headers = await authHeaders(getToken);
            const res = await fetch(`${API_BASE}/keys/${id}`, {
                method: "DELETE",
                headers,
            });
            if (!res.ok) throw new Error("Failed to delete key");
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["apiKeys", user?.id] });
            setDeletingId(null);
        },
    });

    const handleCreate = (e: React.FormEvent) => {
        e.preventDefault();
        if (!newKeyName.trim()) return;
        createMutation.mutate(newKeyName.trim());
    };

    const handleCopy = () => {
        if (!newlyCreatedKey) return;
        navigator.clipboard.writeText(newlyCreatedKey);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    if (!isLoaded || !user) {
        return (
            <div className="flex items-center justify-center min-h-screen">
                <p className="animate-pulse text-muted-foreground text-sm">Loading...</p>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-background text-foreground">
            <div className="max-w-2xl mx-auto px-6 py-16 space-y-10">

                {/* Header */}
                <div className="space-y-1">
                    <h1 className="text-2xl font-semibold tracking-tight">API Keys</h1>
                    <p className="text-sm text-muted-foreground">
                        Manage keys used to authenticate the Atlas CLI with your account.
                    </p>
                </div>

                {/* One-time key reveal banner */}
                {newlyCreatedKey && (
                    <Card className="border-amber-500/20 bg-amber-500/5">
                        <CardContent className="pt-4 space-y-3">
                            <p className="text-xs text-yellow-600 dark:text-yellow-400 font-medium uppercase tracking-widest">
                                Save this key — it will not be shown again
                            </p>
                            <div className="flex items-center gap-2">
                                <code className="flex-1 text-sm font-mono bg-background border border-border rounded-lg px-3 py-2 text-yellow-600 dark:text-yellow-300 overflow-x-auto">
                                    {keyVisible
                                        ? newlyCreatedKey
                                        : `${newlyCreatedKey.slice(0, 8)}${"•".repeat(32)}`}
                                </code>
                                <Button
                                    variant="outline"
                                    size="icon"
                                    onClick={() => setKeyVisible((v) => !v)}
                                    title={keyVisible ? "Hide" : "Reveal"}
                                >
                                    <HugeiconsIcon icon={keyVisible ? ViewOffIcon : EyeIcon} size={16} strokeWidth={2} />
                                </Button>
                                <Button
                                    variant="outline"
                                    size="icon"
                                    onClick={handleCopy}
                                    title="Copy"
                                >
                                    <HugeiconsIcon icon={copied ? Tick01Icon : Copy01Icon} size={16} strokeWidth={2} />
                                </Button>
                            </div>
                            <Button
                                variant="link"
                                size="sm"
                                className="px-0 text-muted-foreground h-auto"
                                onClick={() => setNewlyCreatedKey(null)}
                            >
                                I&apos;ve saved it — dismiss
                            </Button>
                        </CardContent>
                    </Card>
                )}

                {/* Create form */}
                <form onSubmit={handleCreate} className="flex gap-2">
                    <Input
                        type="text"
                        placeholder="Key name (e.g. My Laptop)"
                        value={newKeyName}
                        onChange={(e) => setNewKeyName(e.target.value)}
                        className="flex-1"
                    />
                    <Button
                        type="submit"
                        disabled={!newKeyName.trim() || createMutation.isPending}
                    >
                        <HugeiconsIcon icon={PlusSignIcon} size={15} strokeWidth={2} />
                        {createMutation.isPending ? "Creating…" : "Create"}
                    </Button>
                </form>

                {/* Keys list */}
                <div className="space-y-2">
                    {isLoading ? (
                        <div className="space-y-2">
                            {[1, 2].map((i) => (
                                <div
                                    key={i}
                                    className="h-16 rounded-xl bg-muted animate-pulse"
                                />
                            ))}
                        </div>
                    ) : apiKeys.length === 0 ? (
                        <Card className="border-dashed">
                            <CardContent className="py-10 text-center">
                                <HugeiconsIcon icon={Key01Icon} size={28} className="mx-auto text-muted-foreground mb-3" strokeWidth={2} />
                                <p className="text-sm text-muted-foreground">No API keys yet.</p>
                            </CardContent>
                        </Card>
                    ) : (
                        apiKeys.map((key) => (
                            <Card key={key.id} className="group">
                                <CardContent className="flex items-center justify-between py-3.5">
                                    <div className="flex items-center gap-3 min-w-0">
                                        <div className="w-8 h-8 rounded-lg bg-muted flex items-center justify-center flex-shrink-0">
                                            <HugeiconsIcon icon={Key01Icon} size={15} className="text-muted-foreground" strokeWidth={2} />
                                        </div>
                                        <div className="min-w-0">
                                            <p className="text-sm font-medium truncate">{key.name}</p>
                                            <p className="text-xs text-muted-foreground font-mono">
                                                {key.prefix}•••• · Created {formatDate(key.created_at)}
                                            </p>
                                        </div>
                                    </div>

                                    {deletingId === key.id ? (
                                        <div className="flex items-center gap-2 ml-4 flex-shrink-0">
                                            <span className="text-xs text-destructive">Revoke?</span>
                                            <Button
                                                variant="destructive"
                                                size="sm"
                                                onClick={() => deleteMutation.mutate(key.id)}
                                                disabled={deleteMutation.isPending}
                                            >
                                                {deleteMutation.isPending ? "…" : "Yes"}
                                            </Button>
                                            <Button
                                                variant="outline"
                                                size="sm"
                                                onClick={() => setDeletingId(null)}
                                            >
                                                Cancel
                                            </Button>
                                        </div>
                                    ) : (
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            onClick={() => setDeletingId(key.id)}
                                            className="ml-4 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive flex-shrink-0"
                                            title="Revoke key"
                                        >
                                            <HugeiconsIcon icon={Delete02Icon} size={15} strokeWidth={2} />
                                        </Button>
                                    )}
                                </CardContent>
                            </Card>
                        ))
                    )}
                </div>

                {apiKeys.length > 0 && (
                    <p className="text-xs text-muted-foreground text-center">
                        {apiKeys.length} key{apiKeys.length !== 1 ? "s" : ""} · Hover a key to revoke it
                    </p>
                )}

                {/* Compute Usage Chart */}
                <div className="pt-4 border-t border-border">
                    <ComputeUsageChart />
                </div>
            </div>
        </div>
    );
}
