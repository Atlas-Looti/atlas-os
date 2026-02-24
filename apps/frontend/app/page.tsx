"use client";

import { useSignIn, useSignUp, useUser } from "@clerk/nextjs";
import { useState, FormEvent, useEffect } from "react";
import { useRouter } from "next/navigation";
import { HugeiconsIcon } from "@hugeicons/react";
import { Mailbox01Icon, ArrowRight01Icon, Tick01Icon } from "@hugeicons/core-free-icons";

export default function AuthPage() {
    const { isLoaded: isSignInLoaded, signIn, setActive: setSignInActive } = useSignIn();
    const { isLoaded: isSignUpLoaded, signUp, setActive: setSignUpActive } = useSignUp();
    const { user, isLoaded: isUserLoaded } = useUser();
    const router = useRouter();

    const [email, setEmail] = useState("");
    const [code, setCode] = useState("");
    const [flow, setFlow] = useState<"email" | "otp">("email");
    const [authType, setAuthType] = useState<"signIn" | "signUp">("signIn");

    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState("");

    // Automatically redirect if already signed in
    useEffect(() => {
        if (isUserLoaded && user) {
            router.push("/dashboard");
        }
    }, [isUserLoaded, user, router]);

    if (!isSignInLoaded || !isSignUpLoaded || !isUserLoaded || user) return null;

    const handleEmailSubmit = async (e: FormEvent) => {
        e.preventDefault();
        if (!email) return;

        setIsLoading(true);
        setError("");

        try {
            // 1. Attempt Sign In first
            const signInAttempt = await signIn.create({
                identifier: email,
            });

            // 2. If Sign In succeeds but needs the first factor (OTP)
            if (signInAttempt.status === "needs_first_factor") {
                const firstFactor = signInAttempt.supportedFirstFactors?.find(
                    (ff) => ff.strategy === "email_code"
                );

                if (!firstFactor || !firstFactor.emailAddressId) {
                    setError("Email verification is missing or unsupported for this account.");
                    setIsLoading(false);
                    return;
                }

                await signIn.prepareFirstFactor({
                    strategy: "email_code",
                    emailAddressId: firstFactor.emailAddressId,
                });

                setAuthType("signIn");
                setFlow("otp");
            } else {
                setError("Unexpected sign in status.");
            }
        } catch (err: any) {
            // 3. If user is not found, we caught the error -> initiate Sign Up
            if (err.errors?.[0]?.code === "form_identifier_not_found") {
                try {
                    await signUp.create({
                        emailAddress: email,
                    });

                    await signUp.prepareEmailAddressVerification({
                        strategy: "email_code",
                    });

                    setAuthType("signUp");
                    setFlow("otp");
                } catch (signUpErr: any) {
                    setError(signUpErr.errors?.[0]?.longMessage || "Failed to sign up.");
                }
            } else {
                setError(err.errors?.[0]?.longMessage || "An error occurred.");
            }
        } finally {
            setIsLoading(false);
        }
    };

    const handleOtpSubmit = async (e: FormEvent) => {
        e.preventDefault();
        if (!code) return;

        setIsLoading(true);
        setError("");

        try {
            if (authType === "signIn") {
                const result = await signIn.attemptFirstFactor({
                    strategy: "email_code",
                    code,
                });

                if (result.status === "complete") {
                    await setSignInActive({ session: result.createdSessionId });
                    router.push("/dashboard"); // Redirect to dashboard after login
                } else {
                    setError("Additional verification needed.");
                }
            } else if (authType === "signUp") {
                const result = await signUp.attemptEmailAddressVerification({
                    code,
                });

                if (result.status === "complete") {
                    await setSignUpActive({ session: result.createdSessionId });
                    router.push("/dashboard"); // Redirect to dashboard after login
                } else {
                    setError("Verification incomplete.");
                }
            }
        } catch (err: any) {
            setError(err.errors?.[0]?.longMessage || "Invalid verification code.");
        } finally {
            setIsLoading(false);
        }
    };

    return (
        <div className="min-h-screen bg-neutral-950 text-neutral-50 flex flex-col items-center justify-center p-4">
            <div className="w-full max-w-sm">
                <div className="mb-8 text-center space-y-2">
                    <div className="w-12 h-12 bg-neutral-900 border border-neutral-800 rounded-xl flex items-center justify-center mx-auto mb-4">
                        <HugeiconsIcon icon={Mailbox01Icon} className="w-6 h-6 text-neutral-400" />
                    </div>
                    <h1 className="text-2xl font-medium tracking-tight">
                        {flow === "email" ? "Welcome back" : "Check your email"}
                    </h1>
                    <p className="text-sm text-neutral-400">
                        {flow === "email"
                            ? "Enter your email to sign in or create an account"
                            : `We've sent a 6-digit code to ${email}`}
                    </p>
                </div>

                <div className="bg-neutral-900/50 border border-neutral-800 rounded-2xl p-6 shadow-xl backdrop-blur-sm">
                    {flow === "email" ? (
                        <form onSubmit={handleEmailSubmit} className="space-y-4">
                            <div className="space-y-2">
                                <label htmlFor="email" className="text-sm font-medium text-neutral-300">
                                    Email Address
                                </label>
                                <input
                                    id="email"
                                    type="email"
                                    value={email}
                                    onChange={(e) => setEmail(e.target.value)}
                                    placeholder="name@example.com"
                                    className="w-full bg-neutral-950 border border-neutral-800 rounded-xl px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-neutral-700 focus:border-neutral-700 transition-colors placeholder:text-neutral-600"
                                    required
                                />
                            </div>

                            {error && <p className="text-sm text-red-500">{error}</p>}

                            <button
                                type="submit"
                                disabled={isLoading}
                                className="w-full bg-white text-black hover:bg-neutral-200 font-medium rounded-xl px-4 py-3 text-sm flex items-center justify-center gap-2 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                            >
                                {isLoading ? (
                                    <span className="w-5 h-5 border-2 border-black border-t-transparent rounded-full animate-spin"></span>
                                ) : (
                                    <>
                                        Continue with Email
                                        <HugeiconsIcon icon={ArrowRight01Icon} className="w-4 h-4" />
                                    </>
                                )}
                            </button>
                        </form>
                    ) : (
                        <form onSubmit={handleOtpSubmit} className="space-y-4">
                            <div className="space-y-2">
                                <label htmlFor="code" className="text-sm font-medium text-neutral-300">
                                    Verification Code
                                </label>
                                <input
                                    id="code"
                                    type="text"
                                    value={code}
                                    onChange={(e) => setCode(e.target.value)}
                                    placeholder="000000"
                                    maxLength={6}
                                    className="w-full bg-neutral-950 border border-neutral-800 rounded-xl px-4 py-3 text-center text-xl tracking-widest focus:outline-none focus:ring-2 focus:ring-neutral-700 focus:border-neutral-700 transition-colors placeholder:text-neutral-600"
                                    required
                                />
                            </div>

                            {error && <p className="text-sm text-red-500">{error}</p>}

                            <button
                                type="submit"
                                disabled={isLoading}
                                className="w-full bg-white text-black hover:bg-neutral-200 font-medium rounded-xl px-4 py-3 text-sm flex items-center justify-center gap-2 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                            >
                                {isLoading ? (
                                    <span className="w-5 h-5 border-2 border-black border-t-transparent rounded-full animate-spin"></span>
                                ) : (
                                    <>
                                        Verify & Continue
                                        <HugeiconsIcon icon={Tick01Icon} className="w-4 h-4" />
                                    </>
                                )}
                            </button>

                            <button
                                type="button"
                                onClick={() => setFlow("email")}
                                className="w-full text-center text-sm text-neutral-500 hover:text-neutral-300 transition-colors mt-4"
                            >
                                Use a different email
                            </button>
                        </form>
                    )}
                </div>
            </div>
        </div>
    );
}