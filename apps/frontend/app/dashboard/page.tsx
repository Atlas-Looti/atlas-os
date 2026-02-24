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

    // Fetch API keys — authenticated by Clerk JWT
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

    // Create key
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

    // Delete key
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
            <div className="flex items-center justify-center min-h-screen bg-black text-white">
                <p className="animate-pulse text-neutral-500 text-sm">Loading...</p>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-black text-white">
            <div className="max-w-2xl mx-auto px-6 py-16 space-y-10">

                {/* Header */}
                <div className="space-y-1">
                    <h1 className="text-2xl font-semibold tracking-tight">API Keys</h1>
                    <p className="text-sm text-neutral-500">
                        Manage keys used to authenticate the Atlas CLI with your account.
                    </p>
                </div>

                {/* One-time key reveal banner */}
                {newlyCreatedKey && (
                    <div className="rounded-xl border border-amber-500/30 bg-amber-500/5 p-4 space-y-3">
                        <p className="text-xs text-amber-400 font-medium uppercase tracking-widest">
                            Save this key — it will not be shown again
                        </p>
                        <div className="flex items-center gap-2">
                            <code className="flex-1 text-sm font-mono bg-black/40 border border-neutral-800 rounded-lg px-3 py-2 text-amber-300 overflow-x-auto">
                                {keyVisible
                                    ? newlyCreatedKey
                                    : `${newlyCreatedKey.slice(0, 8)}${"•".repeat(32)}`}
                            </code>
                            <button
                                onClick={() => setKeyVisible((v) => !v)}
                                className="p-2 rounded-lg bg-neutral-900 hover:bg-neutral-800 transition-colors text-neutral-400"
                                title={keyVisible ? "Hide" : "Reveal"}
                            >
                                <HugeiconsIcon icon={keyVisible ? ViewOffIcon : EyeIcon} size={16} />
                            </button>
                            <button
                                onClick={handleCopy}
                                className="p-2 rounded-lg bg-neutral-900 hover:bg-neutral-800 transition-colors text-neutral-400"
                                title="Copy"
                            >
                                <HugeiconsIcon icon={copied ? Tick01Icon : Copy01Icon} size={16} />
                            </button>
                        </div>
                        <button
                            onClick={() => setNewlyCreatedKey(null)}
                            className="text-xs text-neutral-600 hover:text-neutral-400 transition-colors"
                        >
                            I&apos;ve saved it — dismiss
                        </button>
                    </div>
                )}

                {/* Create form */}
                <form onSubmit={handleCreate} className="flex gap-2">
                    <input
                        type="text"
                        placeholder="Key name (e.g. My Laptop)"
                        value={newKeyName}
                        onChange={(e) => setNewKeyName(e.target.value)}
                        className="flex-1 bg-neutral-950 border border-neutral-800 rounded-lg px-4 py-2.5 text-sm text-white placeholder-neutral-600 focus:outline-none focus:border-neutral-600 transition-colors"
                    />
                    <button
                        type="submit"
                        disabled={!newKeyName.trim() || createMutation.isPending}
                        className="flex items-center gap-2 px-4 py-2.5 bg-white text-black text-sm font-medium rounded-lg hover:bg-neutral-200 disabled:opacity-40 disabled:cursor-not-allowed transition-all"
                    >
                        <HugeiconsIcon icon={PlusSignIcon} size={15} />
                        {createMutation.isPending ? "Creating…" : "Create"}
                    </button>
                </form>

                {/* Keys list */}
                <div className="space-y-2">
                    {isLoading ? (
                        <div className="space-y-2">
                            {[1, 2].map((i) => (
                                <div
                                    key={i}
                                    className="h-16 rounded-xl bg-neutral-950 border border-neutral-900 animate-pulse"
                                />
                            ))}
                        </div>
                    ) : apiKeys.length === 0 ? (
                        <div className="rounded-xl border border-dashed border-neutral-800 p-10 text-center">
                            <HugeiconsIcon icon={Key01Icon} size={28} className="mx-auto text-neutral-700 mb-3" />
                            <p className="text-sm text-neutral-600">No API keys yet.</p>
                        </div>
                    ) : (
                        apiKeys.map((key) => (
                            <div
                                key={key.id}
                                className="flex items-center justify-between px-4 py-3.5 rounded-xl bg-neutral-950 border border-neutral-900 hover:border-neutral-800 transition-colors group"
                            >
                                <div className="flex items-center gap-3 min-w-0">
                                    <div className="w-8 h-8 rounded-lg bg-neutral-900 flex items-center justify-center flex-shrink-0">
                                        <HugeiconsIcon icon={Key01Icon} size={15} className="text-neutral-500" />
                                    </div>
                                    <div className="min-w-0">
                                        <p className="text-sm font-medium text-white truncate">{key.name}</p>
                                        <p className="text-xs text-neutral-600 font-mono">
                                            {key.prefix}•••• · Created {formatDate(key.created_at)}
                                        </p>
                                    </div>
                                </div>

                                {deletingId === key.id ? (
                                    <div className="flex items-center gap-2 ml-4 flex-shrink-0">
                                        <span className="text-xs text-red-400">Revoke?</span>
                                        <button
                                            onClick={() => deleteMutation.mutate(key.id)}
                                            disabled={deleteMutation.isPending}
                                            className="text-xs px-2.5 py-1 rounded-md bg-red-500/10 text-red-400 hover:bg-red-500/20 transition-colors disabled:opacity-50"
                                        >
                                            {deleteMutation.isPending ? "…" : "Yes"}
                                        </button>
                                        <button
                                            onClick={() => setDeletingId(null)}
                                            className="text-xs px-2.5 py-1 rounded-md bg-neutral-900 text-neutral-400 hover:bg-neutral-800 transition-colors"
                                        >
                                            Cancel
                                        </button>
                                    </div>
                                ) : (
                                    <button
                                        onClick={() => setDeletingId(key.id)}
                                        className="ml-4 p-2 rounded-lg opacity-0 group-hover:opacity-100 text-neutral-600 hover:text-red-400 hover:bg-red-400/10 transition-all flex-shrink-0"
                                        title="Revoke key"
                                    >
                                        <HugeiconsIcon icon={Delete02Icon} size={15} />
                                    </button>
                                )}
                            </div>
                        ))
                    )}
                </div>

                {apiKeys.length > 0 && (
                    <p className="text-xs text-neutral-700 text-center">
                        {apiKeys.length} key{apiKeys.length !== 1 ? "s" : ""} · Hover a key to revoke it
                    </p>
                )}
            </div>
        </div>
    );
}
