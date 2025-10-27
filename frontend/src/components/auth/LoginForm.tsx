import { useState, type FormEvent } from 'react';
import { useTranslation } from 'react-i18next';
import { useAuthStore } from '../../stores/authStore';
import { Button } from '../common/Button';
import { Input } from '../common/Input';

export const LoginForm: React.FC = () => {
  const { t } = useTranslation();
  const [tenant_id, setTenantId] = useState('');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  
  const { login, isLoading, error, clearError } = useAuthStore();

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    clearError();
    
    try {
      await login({ tenant_id, username, password });
    } catch (err) {
      // Error is handled by the store
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      <div>
        <label htmlFor="tenant_id" className="block text-sm font-medium text-gray-700 mb-2">
          {t('common.tenant')} ID
        </label>
        <Input
          id="tenant_id"
          type="text"
          value={tenant_id}
          onChange={(e) => setTenantId(e.target.value)}
          placeholder="Enter your tenant ID"
          required
          disabled={isLoading}
        />
      </div>

      <div>
        <label htmlFor="username" className="block text-sm font-medium text-gray-700 mb-2">
          {t('auth.username')}
        </label>
        <Input
          id="username"
          type="text"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
          placeholder={t('auth.username')}
          required
          disabled={isLoading}
        />
      </div>

      <div>
        <label htmlFor="password" className="block text-sm font-medium text-gray-700 mb-2">
          {t('auth.password')}
        </label>
        <Input
          id="password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          placeholder={t('auth.password')}
          required
          disabled={isLoading}
        />
      </div>

      {error && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-lg text-red-700 text-sm">
          {error}
        </div>
      )}

      <Button
        type="submit"
        variant="primary"
        fullWidth
        disabled={isLoading}
        isLoading={isLoading}
      >
        {t('auth.login')}
      </Button>
    </form>
  );
};
