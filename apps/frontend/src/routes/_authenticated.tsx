import { createFileRoute, redirect } from '@tanstack/react-router'

export const Route = createFileRoute('/_authenticated')({
  beforeLoad: async ({ context }) => {
    const { user } = context

    if (!user?.session) {
      return redirect({ to: '/auth/login' })
    }

    return {
      user,
    }
  },
})
