import { useMutation } from '@tanstack/react-query'
import { useRouter } from '@tanstack/react-router'
import { Fingerprint, LoaderCircle } from 'lucide-react'
import * as React from 'react'
import { toast } from 'sonner'

import { Button } from '~/lib/components/ui/button'
import { Input } from '~/lib/components/ui/input'
import { Label } from '~/lib/components/ui/label'
import { cn } from '~/lib/utils/cn'
import { authClient } from '~/lib/auth-client'

type UserAuthFormProps = React.HTMLAttributes<HTMLDivElement>

export function UserAuthFormRegister({ className, ...props }: UserAuthFormProps) {
  const [name, setName] = React.useState('')
  const [email, setEmail] = React.useState('')
  const [isRegistered, setIsRegistered] = React.useState(false)
  const router = useRouter()

  const registerMutation = useMutation({
    mutationFn: async (variables: { name: string; email: string }) => {
      const randomPassword = crypto.randomUUID() + crypto.randomUUID()
      const { data, error } = await authClient.signUp.email({
        name: variables.name,
        email: variables.email,
        password: randomPassword,
      })
      if (error) throw new Error(error.message)
      return data
    },
    onSuccess: () => {
      setIsRegistered(true)
      toast.success('Account created -- now register your passkey')
    },
    onError: (error) => {
      toast.error(`Registration failed: ${error.message}`)
    },
  })

  const addPasskeyMutation = useMutation({
    mutationFn: async () => {
      const { data, error } = await authClient.passkey.addPasskey({
        name: `${name}'s passkey`,
      })
      if (error) throw new Error(error.message)
      return data
    },
    onSuccess: () => {
      toast.success('Passkey registered -- you\'re all set!')
      router.navigate({ to: '/' })
    },
    onError: (error) => {
      toast.error(`Passkey registration failed: ${error.message}`)
    },
  })

  const error =
    registerMutation.error?.message || addPasskeyMutation.error?.message || null

  const isLoading = registerMutation.isPending || addPasskeyMutation.isPending

  async function onSubmit(event: React.SyntheticEvent) {
    event.preventDefault()
    registerMutation.mutate({ name, email })
  }

  return (
    <div className={cn('grid gap-6', className)} {...props}>
      {error && <div className="rounded-md bg-red-50 p-3 text-sm text-red-500">{error}</div>}

      {!isRegistered ? (
        <form onSubmit={onSubmit}>
          <div className="grid gap-2">
            <div className="grid gap-1">
              <Label className="sr-only" htmlFor="name">
                Name
              </Label>
              <Input
                id="name"
                placeholder="Full name"
                type="text"
                autoCapitalize="words"
                autoComplete="name"
                disabled={isLoading}
                value={name}
                onChange={(e) => setName(e.target.value)}
                required
              />
            </div>
            <div className="grid gap-1">
              <Label className="sr-only" htmlFor="email">
                Email
              </Label>
              <Input
                id="email"
                placeholder="name@example.com"
                type="email"
                autoCapitalize="none"
                autoComplete="email"
                autoCorrect="off"
                disabled={isLoading}
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
              />
            </div>
            <Button disabled={isLoading} type="submit">
              {registerMutation.isPending && (
                <LoaderCircle className="mr-2 h-4 w-4 animate-spin" />
              )}
              Continue
            </Button>
          </div>
        </form>
      ) : (
        <div className="grid gap-4">
          <div className="rounded-md bg-green-50 p-3 text-sm text-green-600">
            Account created! Register a passkey to sign in.
          </div>
          <Button
            size="lg"
            type="button"
            disabled={isLoading}
            onClick={() => addPasskeyMutation.mutate()}
          >
            {addPasskeyMutation.isPending ? (
              <LoaderCircle className="mr-2 h-5 w-5 animate-spin" />
            ) : (
              <Fingerprint className="mr-2 h-5 w-5" />
            )}
            Register Passkey
          </Button>
        </div>
      )}
    </div>
  )
}
