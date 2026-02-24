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

    // 1. Fetch modules (Status Atlas OS) using React Query
    const { data: modules = [], isLoading: isModulesLoading } = useQuery({
        queryKey: ["modules"],
        queryFn: async () => {
            const response = await fetch(`http://localhost:3001/api/modules`);
            if (!response.ok) throw new Error("Failed to fetch modules");
            const data = await response.json();
            return data.modules;
        },
        enabled: isLoaded && !!user,
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
                        <h1 className="text-3xl font-semibold mb-2">System Status</h1>
                        <p className="text-neutral-400">
                            Atlas OS active protocols and connection status.
                        </p>
                    </div>
                </div>

                {/* Modules List */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {isModulesLoading ? (
                        <p className="text-neutral-500">Loading modules...</p>
                    ) : (
                        modules.map((mod: any) => (
                            <div key={mod.name} className="p-6 bg-neutral-900 border border-neutral-800 rounded-2xl space-y-3">
                                <div className="flex justify-between items-center">
                                    <h3 className="text-lg font-medium capitalize">{mod.name}</h3>
                                    <span className={`px-3 py-1 rounded-full text-xs font-medium ${mod.enabled ? 'bg-green-500/10 text-green-400' : 'bg-red-500/10 text-red-400'}`}>
                                        {mod.enabled ? 'Active' : 'Disabled'}
                                    </span>
                                </div>
                                <div className="text-sm text-neutral-400 space-y-1">
                                    <p><span className="text-neutral-500">Type:</span> {mod.type}</p>
                                    {mod.config.network && <p><span className="text-neutral-500">Network:</span> {mod.config.network}</p>}
                                    {mod.config.chain && <p><span className="text-neutral-500">Chain:</span> {mod.config.chain}</p>}
                                </div>
                            </div>
                        ))
                    )}
                </div>
            </div>
        </div>
    );
}
