"use client";

import { useUser } from "@clerk/nextjs";
import { useState } from "react";
import { HugeiconsIcon } from "@hugeicons/react";
import {
    PlusSignIcon,
    Delete02Icon,
    Copy01Icon,
    Tick01Icon,
    Key01Icon,
} from "@hugeicons/core-free-icons";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

// Types corresponding to our Rust backend
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

export default function DashboardPage() {
    const { user, isLoaded } = useUser();
    const queryClient = useQueryClient();

    const [newKeyName, setNewKeyName] = useState("");
    const [newlyCreatedKey, setNewlyCreatedKey] = useState<string | null>(null);
    const [copiedKey, setCopiedKey] = useState(false);

    // 1. Fetch keys using React Query
    const { data: keys = [], isLoading } = useQuery<ApiKey[]>({
        queryKey: ["apiKeys", user?.id],
        queryFn: async () => {
            const response = await fetch(`http://localhost:8080/api/keys?user_id=${user?.id}`);
            if (!response.ok) throw new Error("Failed to fetch keys");
            return response.json();
        },
        enabled: isLoaded && !!user, // Only run query when user is logged in
    });

    // 2. Create Key Mutation
    const createKeyMutation = useMutation({
        mutationFn: async (name: string) => {
            if (!user) throw new Error("No user");
            const response = await fetch("http://localhost:8080/api/keys", {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ user_id: user.id, name }),
            });
            if (!response.ok) throw new Error("Failed to create key");
            return response.json() as Promise<CreateKeyResponse>;
        },
        onSuccess: (data) => {
            // Optimistically update or invalidate the query
            queryClient.invalidateQueries({ queryKey: ["apiKeys", user?.id] });
            setNewlyCreatedKey(data.key);
            setNewKeyName("");
        },
    });

    // 3. Delete Key Mutation
    const deleteKeyMutation = useMutation({
        mutationFn: async (id: string) => {
            const response = await fetch(`http://localhost:8080/api/keys/${id}`, {
                method: "DELETE",
            });
            if (!response.ok) throw new Error("Failed to delete key");
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["apiKeys", user?.id] });
        },
    });

    const handleCreateKey = (e: React.FormEvent) => {
        e.preventDefault();
        if (!newKeyName.trim() || !user) return;
        createKeyMutation.mutate(newKeyName.trim());
    };

    const handleDeleteKey = (id: string) => {
        deleteKeyMutation.mutate(id);
    };

    const copyToClipboard = () => {
        if (newlyCreatedKey) {
            navigator.clipboard.writeText(newlyCreatedKey);
            setCopiedKey(true);
            setTimeout(() => setCopiedKey(false), 2000);
        }
    };

    if (!isLoaded || !user) {
        return (
            <div className="flex items-center justify-center min-h-screen bg-black text-white">
                <p className="animate-pulse">Loading dashboard...</p>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-black text-white p-8 md:p-16">
            <div className="max-w-4xl mx-auto space-y-12">
                <div className="flex justify-between items-end border-b border-neutral-800 pb-6">
                    <div>
                        <h1 className="text-3xl font-semibold mb-2">API Keys</h1>
                        <p className="text-neutral-400">
                            Manage your secret API keys to authenticate requests to the Atlas API.
                        </p>
                    </div>
                </div>

                {/* New Key Alert */}
                {newlyCreatedKey && (
                    <div className="p-6 bg-neutral-900 border border-green-900/50 rounded-2xl space-y-4">
                        <div className="flex items-center gap-3 text-green-400">
                            <HugeiconsIcon icon={Tick01Icon} className="w-5 h-5" />
                            <h3 className="font-medium">New key created successfully</h3>
                        </div>
                        <p className="text-sm text-neutral-400">
                            Please copy this key now. For your security, it will never be shown again.
                        </p>
                        <div className="flex items-center gap-2">
                            <code className="flex-1 p-3 bg-black border border-neutral-800 rounded-xl font-mono text-sm text-green-300 break-all">
                                {newlyCreatedKey}
                            </code>
                            <button
                                onClick={copyToClipboard}
                                className="p-3 bg-neutral-800 hover:bg-neutral-700 text-white rounded-xl transition-colors shrink-0"
                            >
                                <HugeiconsIcon
                                    icon={copiedKey ? Tick01Icon : Copy01Icon}
                                    className="w-5 h-5"
                                />
                            </button>
                        </div>
                        <button
                            onClick={() => setNewlyCreatedKey(null)}
                            className="text-sm text-neutral-500 hover:text-white transition-colors"
                        >
                            I have saved my key
                        </button>
                    </div>
                )}

                {/* Create Key Form */}
                <form
                    onSubmit={handleCreateKey}
                    className="flex flex-col sm:flex-row gap-4 p-6 bg-neutral-900 border border-neutral-800 rounded-2xl"
                >
                    <div className="flex-1">
                        <label htmlFor="keyName" className="sr-only">
                            Key Name
                        </label>
                        <input
                            id="keyName"
                            type="text"
                            placeholder="e.g. Production Key, Development Mac..."
                            value={newKeyName}
                            onChange={(e) => setNewKeyName(e.target.value)}
                            className="w-full bg-black border border-neutral-800 rounded-xl px-4 py-3 text-sm focus:outline-none focus:border-neutral-600 focus:ring-1 focus:ring-neutral-600 transition-all placeholder:text-neutral-600"
                            required
                        />
                    </div>
                    <button
                        type="submit"
                        disabled={createKeyMutation.isPending || !newKeyName.trim()}
                        className="flex items-center justify-center gap-2 bg-white text-black px-6 py-3 rounded-xl text-sm font-medium hover:bg-neutral-200 transition-colors disabled:opacity-50 disabled:cursor-not-allowed whitespace-nowrap"
                    >
                        <HugeiconsIcon icon={PlusSignIcon} className="w-4 h-4" />
                        {createKeyMutation.isPending ? "Creating..." : "Create new key"}
                    </button>
                </form>

                {/* Keys List */}
                <div className="border border-neutral-800 rounded-2xl overflow-hidden bg-neutral-900/50">
                    <table className="w-full text-left text-sm">
                        <thead className="bg-neutral-900 border-b border-neutral-800">
                            <tr>
                                <th className="px-6 py-4 font-medium text-neutral-400">Name</th>
                                <th className="px-6 py-4 font-medium text-neutral-400">Key Prefix</th>
                                <th className="px-6 py-4 font-medium text-neutral-400">Created</th>
                                <th className="px-6 py-4 font-medium text-neutral-400 text-right">
                                    Action
                                </th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-neutral-800">
                            {isLoading ? (
                                <tr>
                                    <td colSpan={4} className="px-6 py-8 text-center text-neutral-500">
                                        Loading keys...
                                    </td>
                                </tr>
                            ) : keys.length === 0 ? (
                                <tr>
                                    <td colSpan={4} className="px-6 py-12 text-center">
                                        <div className="flex flex-col items-center gap-3">
                                            <HugeiconsIcon
                                                icon={Key01Icon}
                                                className="w-8 h-8 text-neutral-600"
                                            />
                                            <p className="text-neutral-400">
                                                You don't have any API keys yet.
                                            </p>
                                        </div>
                                    </td>
                                </tr>
                            ) : (
                                keys.map((key) => (
                                    <tr key={key.id} className="group hover:bg-neutral-900 transition-colors">
                                        <td className="px-6 py-4 font-medium">{key.name}</td>
                                        <td className="px-6 py-4 font-mono text-neutral-400">
                                            {key.prefix}...
                                        </td>
                                        <td className="px-6 py-4 text-neutral-500">
                                            {new Date(key.created_at).toLocaleDateString(undefined, {
                                                year: "numeric",
                                                month: "short",
                                                day: "numeric",
                                            })}
                                        </td>
                                        <td className="px-6 py-4 text-right">
                                            <button
                                                onClick={() => handleDeleteKey(key.id)}
                                                disabled={deleteKeyMutation.isPending && deleteKeyMutation.variables === key.id}
                                                className="p-2 text-neutral-500 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-colors opacity-0 group-hover:opacity-100 disabled:opacity-50 disabled:cursor-not-allowed focus:opacity-100"
                                                title="Revoke Key"
                                            >
                                                <HugeiconsIcon icon={Delete02Icon} className="w-4 h-4" />
                                            </button>
                                        </td>
                                    </tr>
                                ))
                            )}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    );
}
