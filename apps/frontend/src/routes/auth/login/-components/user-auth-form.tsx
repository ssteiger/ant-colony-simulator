import { useMutation } from '@tanstack/react-query'
import { Fingerprint, LoaderCircle } from 'lucide-react'
import { useRouter } from '@tanstack/react-router'
import * as React from 'react'
import { toast } from 'sonner'

import { Button } from '~/lib/components/ui/button'
import { cn } from '~/lib/utils/cn'
import { authClient } from '~/lib/auth-client'

type UserAuthFormProps = React.HTMLAttributes<HTMLDivElement>

export function UserAuthFormLogin({ className, ...props }: UserAuthFormProps) {
  const router = useRouter()

  const passkeyMutation = useMutation({
    mutationFn: async () => {
      const { data, error } = await authClient.signIn.passkey()
      if (error) throw new Error(error.message)
      return data
    },
    onSuccess: () => {
      toast.success('Login successful')
      router.navigate({ to: '/' })
    },
    onError: (error) => {
      toast.error(`Passkey login failed: ${error.message}`)
    },
  })

  return (
    <div className={cn('grid gap-6', className)} {...props}>
      {passkeyMutation.error && (
        <div className="rounded-md bg-red-50 p-3 text-sm text-red-500">
          {passkeyMutation.error.message}
        </div>
      )}

      <Button
        type="button"
        size="lg"
        disabled={passkeyMutation.isPending}
        onClick={() => passkeyMutation.mutate()}
      >
        {passkeyMutation.isPending ? (
          <LoaderCircle className="mr-2 h-5 w-5 animate-spin" />
        ) : (
          <Fingerprint className="mr-2 h-5 w-5" />
        )}
        Sign in with Passkey
      </Button>
    </div>
  )
}
